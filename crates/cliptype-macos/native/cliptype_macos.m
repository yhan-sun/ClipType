#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#import <Carbon/Carbon.h>
#import <CoreGraphics/CoreGraphics.h>
#import <Foundation/Foundation.h>
#import <ServiceManagement/ServiceManagement.h>

#include <stdint.h>
#include <stdlib.h>
#include <string.h>

typedef void (*CTCommandCallback)(int command, void *context);

enum {
    CT_COMMAND_TRIGGER = 1,
    CT_COMMAND_CANCEL = 2,
    CT_COMMAND_OPEN_SETTINGS = 3,
    CT_COMMAND_TOGGLE_ENABLED = 4,
    CT_COMMAND_TOGGLE_STARTUP = 5,
    CT_COMMAND_PERMISSION = 6,
    CT_COMMAND_ABOUT = 7,
    CT_COMMAND_QUIT = 8,
};

enum {
    CT_CLIPBOARD_OK = 0,
    CT_CLIPBOARD_EMPTY = 1,
    CT_CLIPBOARD_NON_TEXT = 2,
    CT_CLIPBOARD_MALFORMED = 3,
    CT_CLIPBOARD_CHANGED = 4,
    CT_CLIPBOARD_ALLOCATION = 5,
};

enum {
    CT_HOTKEY_OK = 0,
    CT_HOTKEY_CONFLICT = 1,
    CT_HOTKEY_UNSUPPORTED = 2,
    CT_HOTKEY_NATIVE = 3,
};

enum {
    CT_STARTUP_NOT_REGISTERED = 0,
    CT_STARTUP_ENABLED = 1,
    CT_STARTUP_REQUIRES_APPROVAL = 2,
    CT_STARTUP_NOT_FOUND = 3,
    CT_STARTUP_UNSUPPORTED = 4,
    CT_STARTUP_UNKNOWN = 5,
};

static UInt32 ct_carbon_modifiers(uint8_t modifiers) {
    UInt32 native = 0;
    if ((modifiers & (1u << 0)) != 0) native |= controlKey;
    if ((modifiers & (1u << 1)) != 0) native |= optionKey;
    if ((modifiers & (1u << 2)) != 0) native |= shiftKey;
    if ((modifiers & (1u << 3)) != 0) native |= cmdKey;
    return native;
}

void ct_macos_initialize_application(void) {
    @autoreleasepool {
        [NSApplication sharedApplication];
        [NSApp setActivationPolicy:NSApplicationActivationPolicyAccessory];
    }
}

int64_t ct_macos_clipboard_change_count(void) {
    @autoreleasepool {
        return (int64_t)[[NSPasteboard generalPasteboard] changeCount];
    }
}

int ct_macos_clipboard_copy_utf8(
    uint8_t **out_bytes,
    size_t *out_length,
    int64_t *out_revision
) {
    if (out_bytes == NULL || out_length == NULL || out_revision == NULL) {
        return CT_CLIPBOARD_MALFORMED;
    }
    *out_bytes = NULL;
    *out_length = 0;
    *out_revision = -1;

    @autoreleasepool {
        NSPasteboard *pasteboard = [NSPasteboard generalPasteboard];
        NSInteger before = pasteboard.changeCount;
        NSString *text = [pasteboard stringForType:NSPasteboardTypeString];
        if (text == nil) {
            return CT_CLIPBOARD_NON_TEXT;
        }
        NSData *data = [text dataUsingEncoding:NSUTF8StringEncoding allowLossyConversion:NO];
        if (data == nil) {
            return CT_CLIPBOARD_MALFORMED;
        }
        if (data.length == 0) {
            return CT_CLIPBOARD_EMPTY;
        }
        void *buffer = malloc(data.length);
        if (buffer == NULL) {
            return CT_CLIPBOARD_ALLOCATION;
        }
        memcpy(buffer, data.bytes, data.length);
        NSInteger after = pasteboard.changeCount;
        if (before != after) {
            free(buffer);
            return CT_CLIPBOARD_CHANGED;
        }
        *out_bytes = (uint8_t *)buffer;
        *out_length = data.length;
        *out_revision = (int64_t)after;
        return CT_CLIPBOARD_OK;
    }
}

void ct_macos_free(void *pointer) {
    free(pointer);
}

int ct_macos_accessibility_trusted(void) {
    return AXIsProcessTrusted() ? 1 : 0;
}

void ct_macos_request_accessibility(void) {
    @autoreleasepool {
        NSDictionary *options = @{
            (__bridge NSString *)kAXTrustedCheckOptionPrompt: @YES,
        };
        (void)AXIsProcessTrustedWithOptions((__bridge CFDictionaryRef)options);
    }
}

int ct_macos_open_accessibility_settings(void) {
    @autoreleasepool {
        NSURL *url = [NSURL URLWithString:
            @"x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"];
        if (url == nil) return 0;
        return [[NSWorkspace sharedWorkspace] openURL:url] ? 1 : 0;
    }
}

int ct_macos_capture_target(
    int32_t *out_process_id,
    uint64_t *out_focus_hash,
    int *out_focus_available
) {
    if (out_process_id == NULL || out_focus_hash == NULL || out_focus_available == NULL) {
        return 0;
    }
    *out_process_id = 0;
    *out_focus_hash = 0;
    *out_focus_available = 0;

    @autoreleasepool {
        NSRunningApplication *application =
            [[NSWorkspace sharedWorkspace] frontmostApplication];
        if (application == nil || application.processIdentifier <= 0) {
            return 0;
        }
        *out_process_id = application.processIdentifier;

        if (!AXIsProcessTrusted()) {
            return 1;
        }

        AXUIElementRef system = AXUIElementCreateSystemWide();
        if (system == NULL) return 1;
        CFTypeRef focused = NULL;
        AXError error = AXUIElementCopyAttributeValue(
            system,
            kAXFocusedUIElementAttribute,
            &focused
        );
        CFRelease(system);
        if (error != kAXErrorSuccess || focused == NULL) {
            if (focused != NULL) CFRelease(focused);
            return 1;
        }

        pid_t focus_pid = 0;
        if (AXUIElementGetPid((AXUIElementRef)focused, &focus_pid) == kAXErrorSuccess &&
            focus_pid == *out_process_id) {
            *out_focus_hash = (uint64_t)CFHash(focused);
            *out_focus_available = 1;
        }
        CFRelease(focused);
        return 1;
    }
}

uint64_t ct_macos_modifier_flags(void) {
    return (uint64_t)CGEventSourceFlagsState(kCGEventSourceStateCombinedSessionState);
}

int ct_macos_secure_input_enabled(void) {
    return IsSecureEventInputEnabled() ? 1 : 0;
}

static int ct_post_balanced_key(CGKeyCode keycode, CGEventFlags flags) {
    if (!AXIsProcessTrusted() || IsSecureEventInputEnabled()) return 0;
    CGEventSourceRef source = CGEventSourceCreate(kCGEventSourceStateCombinedSessionState);
    if (source == NULL) return 0;
    CGEventRef down = CGEventCreateKeyboardEvent(source, keycode, true);
    CGEventRef up = CGEventCreateKeyboardEvent(source, keycode, false);
    if (down == NULL || up == NULL) {
        if (down != NULL) CFRelease(down);
        if (up != NULL) CFRelease(up);
        CFRelease(source);
        return 0;
    }
    CGEventSetFlags(down, flags);
    CGEventSetFlags(up, flags);
    CGEventPost(kCGHIDEventTap, down);
    CGEventPost(kCGHIDEventTap, up);
    CFRelease(down);
    CFRelease(up);
    CFRelease(source);
    return 1;
}

int ct_macos_post_unicode(const uint16_t *units, size_t length) {
    if (units == NULL || length == 0 || length > 32) return 0;
    if (!AXIsProcessTrusted() || IsSecureEventInputEnabled()) return 0;
    CGEventSourceRef source = CGEventSourceCreate(kCGEventSourceStateCombinedSessionState);
    if (source == NULL) return 0;
    CGEventRef down = CGEventCreateKeyboardEvent(source, (CGKeyCode)0, true);
    CGEventRef up = CGEventCreateKeyboardEvent(source, (CGKeyCode)0, false);
    if (down == NULL || up == NULL) {
        if (down != NULL) CFRelease(down);
        if (up != NULL) CFRelease(up);
        CFRelease(source);
        return 0;
    }
    CGEventKeyboardSetUnicodeString(down, (UniCharCount)length, (const UniChar *)units);
    CGEventPost(kCGHIDEventTap, down);
    CGEventPost(kCGHIDEventTap, up);
    CFRelease(down);
    CFRelease(up);
    CFRelease(source);
    return 1;
}

int ct_macos_post_return(void) {
    return ct_post_balanced_key((CGKeyCode)36, 0);
}

int ct_macos_post_tab(void) {
    return ct_post_balanced_key((CGKeyCode)48, 0);
}

int ct_macos_post_backspace(void) {
    return ct_post_balanced_key((CGKeyCode)51, 0);
}

int ct_macos_post_cursor_right(void) {
    return ct_post_balanced_key((CGKeyCode)124, 0);
}

int ct_macos_post_paste(int64_t expected_revision) {
    if (ct_macos_clipboard_change_count() != expected_revision) return -1;
    return ct_post_balanced_key((CGKeyCode)9, kCGEventFlagMaskCommand);
}

typedef struct {
    EventHotKeyRef trigger;
    EventHotKeyRef cancel;
    EventHandlerRef handler;
    UInt32 trigger_id;
    UInt32 cancel_id;
    uint16_t trigger_code;
    uint16_t cancel_code;
    uint8_t trigger_modifiers;
    uint8_t cancel_modifiers;
    UInt32 next_id;
    CTCommandCallback callback;
    void *context;
} CTHotkeyController;

static OSStatus ct_hotkey_event_handler(
    EventHandlerCallRef next_handler,
    EventRef event,
    void *context
) {
    (void)next_handler;
    CTHotkeyController *controller = (CTHotkeyController *)context;
    EventHotKeyID identifier = {0};
    OSStatus status = GetEventParameter(
        event,
        kEventParamDirectObject,
        typeEventHotKeyID,
        NULL,
        sizeof(identifier),
        NULL,
        &identifier
    );
    if (status != noErr || controller == NULL || controller->callback == NULL) {
        return status;
    }
    if (identifier.id == controller->trigger_id) {
        controller->callback(CT_COMMAND_TRIGGER, controller->context);
    } else if (identifier.id == controller->cancel_id) {
        controller->callback(CT_COMMAND_CANCEL, controller->context);
    }
    return noErr;
}

void *ct_macos_hotkey_create(CTCommandCallback callback, void *context) {
    CTHotkeyController *controller = calloc(1, sizeof(CTHotkeyController));
    if (controller == NULL) return NULL;
    controller->callback = callback;
    controller->context = context;
    controller->next_id = 100;
    EventTypeSpec type = { kEventClassKeyboard, kEventHotKeyPressed };
    OSStatus status = InstallApplicationEventHandler(
        NewEventHandlerUPP(ct_hotkey_event_handler),
        1,
        &type,
        controller,
        &controller->handler
    );
    if (status != noErr) {
        free(controller);
        return NULL;
    }
    return controller;
}

static int ct_register_one(
    uint16_t code,
    uint8_t modifiers,
    UInt32 identifier,
    EventHotKeyRef *out_ref
) {
    EventHotKeyID hotkey_id = { 'ClTp', identifier };
    OSStatus status = RegisterEventHotKey(
        (UInt32)code,
        ct_carbon_modifiers(modifiers),
        hotkey_id,
        GetApplicationEventTarget(),
        0,
        out_ref
    );
    if (status == noErr) return CT_HOTKEY_OK;
    if (status == eventHotKeyExistsErr || status == eventHotKeyInvalidErr) {
        return CT_HOTKEY_CONFLICT;
    }
    return CT_HOTKEY_NATIVE;
}

int ct_macos_hotkey_register_initial(
    void *pointer,
    uint16_t trigger_code,
    uint8_t trigger_modifiers,
    uint16_t cancel_code,
    uint8_t cancel_modifiers
) {
    CTHotkeyController *controller = pointer;
    if (controller == NULL || controller->trigger != NULL || controller->cancel != NULL) {
        return CT_HOTKEY_NATIVE;
    }
    UInt32 trigger_id = controller->next_id++;
    UInt32 cancel_id = controller->next_id++;
    EventHotKeyRef trigger = NULL;
    EventHotKeyRef cancel = NULL;
    int status = ct_register_one(trigger_code, trigger_modifiers, trigger_id, &trigger);
    if (status != CT_HOTKEY_OK) return status;
    status = ct_register_one(cancel_code, cancel_modifiers, cancel_id, &cancel);
    if (status != CT_HOTKEY_OK) {
        UnregisterEventHotKey(trigger);
        return status;
    }
    controller->trigger = trigger;
    controller->cancel = cancel;
    controller->trigger_id = trigger_id;
    controller->cancel_id = cancel_id;
    controller->trigger_code = trigger_code;
    controller->cancel_code = cancel_code;
    controller->trigger_modifiers = trigger_modifiers;
    controller->cancel_modifiers = cancel_modifiers;
    return CT_HOTKEY_OK;
}

static int ct_same_hotkey(uint16_t left_code, uint8_t left_modifiers,
                          uint16_t right_code, uint8_t right_modifiers) {
    return left_code == right_code && left_modifiers == right_modifiers;
}

int ct_macos_hotkey_probe_pair(
    void *pointer,
    uint16_t trigger_code,
    uint8_t trigger_modifiers,
    uint16_t cancel_code,
    uint8_t cancel_modifiers
) {
    CTHotkeyController *controller = pointer;
    if (controller == NULL) return CT_HOTKEY_NATIVE;
    int trigger_same = ct_same_hotkey(
        trigger_code, trigger_modifiers,
        controller->trigger_code, controller->trigger_modifiers
    );
    int cancel_same = ct_same_hotkey(
        cancel_code, cancel_modifiers,
        controller->cancel_code, controller->cancel_modifiers
    );
    int cross_swap = ct_same_hotkey(
        trigger_code, trigger_modifiers,
        controller->cancel_code, controller->cancel_modifiers
    ) || ct_same_hotkey(
        cancel_code, cancel_modifiers,
        controller->trigger_code, controller->trigger_modifiers
    );
    if (cross_swap && !(trigger_same && cancel_same)) return CT_HOTKEY_UNSUPPORTED;

    EventHotKeyRef trigger = NULL;
    EventHotKeyRef cancel = NULL;
    int status = CT_HOTKEY_OK;
    if (!trigger_same) {
        status = ct_register_one(trigger_code, trigger_modifiers, controller->next_id++, &trigger);
        if (status != CT_HOTKEY_OK) return status;
    }
    if (!cancel_same) {
        status = ct_register_one(cancel_code, cancel_modifiers, controller->next_id++, &cancel);
        if (status != CT_HOTKEY_OK) {
            if (trigger != NULL) UnregisterEventHotKey(trigger);
            return status;
        }
    }
    if (trigger != NULL) UnregisterEventHotKey(trigger);
    if (cancel != NULL) UnregisterEventHotKey(cancel);
    return CT_HOTKEY_OK;
}

int ct_macos_hotkey_replace_pair(
    void *pointer,
    uint16_t trigger_code,
    uint8_t trigger_modifiers,
    uint16_t cancel_code,
    uint8_t cancel_modifiers
) {
    CTHotkeyController *controller = pointer;
    if (controller == NULL) return CT_HOTKEY_NATIVE;
    int trigger_same = ct_same_hotkey(
        trigger_code, trigger_modifiers,
        controller->trigger_code, controller->trigger_modifiers
    );
    int cancel_same = ct_same_hotkey(
        cancel_code, cancel_modifiers,
        controller->cancel_code, controller->cancel_modifiers
    );
    if (trigger_same && cancel_same) return CT_HOTKEY_OK;
    if (ct_same_hotkey(trigger_code, trigger_modifiers,
                       controller->cancel_code, controller->cancel_modifiers) ||
        ct_same_hotkey(cancel_code, cancel_modifiers,
                       controller->trigger_code, controller->trigger_modifiers)) {
        return CT_HOTKEY_UNSUPPORTED;
    }

    EventHotKeyRef new_trigger = NULL;
    EventHotKeyRef new_cancel = NULL;
    UInt32 new_trigger_id = controller->trigger_id;
    UInt32 new_cancel_id = controller->cancel_id;
    int status = CT_HOTKEY_OK;
    if (!trigger_same) {
        new_trigger_id = controller->next_id++;
        status = ct_register_one(trigger_code, trigger_modifiers, new_trigger_id, &new_trigger);
        if (status != CT_HOTKEY_OK) return status;
    }
    if (!cancel_same) {
        new_cancel_id = controller->next_id++;
        status = ct_register_one(cancel_code, cancel_modifiers, new_cancel_id, &new_cancel);
        if (status != CT_HOTKEY_OK) {
            if (new_trigger != NULL) UnregisterEventHotKey(new_trigger);
            return status;
        }
    }

    if (!trigger_same && controller->trigger != NULL) UnregisterEventHotKey(controller->trigger);
    if (!cancel_same && controller->cancel != NULL) UnregisterEventHotKey(controller->cancel);
    if (!trigger_same) {
        controller->trigger = new_trigger;
        controller->trigger_id = new_trigger_id;
        controller->trigger_code = trigger_code;
        controller->trigger_modifiers = trigger_modifiers;
    }
    if (!cancel_same) {
        controller->cancel = new_cancel;
        controller->cancel_id = new_cancel_id;
        controller->cancel_code = cancel_code;
        controller->cancel_modifiers = cancel_modifiers;
    }
    return CT_HOTKEY_OK;
}

void ct_macos_hotkey_destroy(void *pointer) {
    CTHotkeyController *controller = pointer;
    if (controller == NULL) return;
    if (controller->trigger != NULL) UnregisterEventHotKey(controller->trigger);
    if (controller->cancel != NULL) UnregisterEventHotKey(controller->cancel);
    if (controller->handler != NULL) RemoveEventHandler(controller->handler);
    free(controller);
}

@interface CTStatusController : NSObject
@property(nonatomic, assign) CTCommandCallback callback;
@property(nonatomic, assign) void *context;
@property(nonatomic, strong) NSStatusItem *statusItem;
@property(nonatomic, strong) NSMenuItem *enabledItem;
@property(nonatomic, strong) NSMenuItem *modeItem;
@property(nonatomic, strong) NSMenuItem *permissionItem;
@property(nonatomic, strong) NSMenuItem *startupItem;
@end

@implementation CTStatusController
- (instancetype)initWithCallback:(CTCommandCallback)callback context:(void *)context {
    self = [super init];
    if (self == nil) return nil;
    _callback = callback;
    _context = context;
    _statusItem = [[NSStatusBar systemStatusBar] statusItemWithLength:NSSquareStatusItemLength];
    NSStatusBarButton *button = _statusItem.button;
    NSString *statusPath = [[NSBundle mainBundle]
        pathForResource:@"ClipTypeStatusTemplate" ofType:@"svg"];
    NSImage *image = statusPath == nil ? nil : [[NSImage alloc] initWithContentsOfFile:statusPath];
    if (image == nil) {
        image = [NSImage imageWithSystemSymbolName:@"doc.on.clipboard"
                         accessibilityDescription:@"ClipType"];
    }
    image.template = YES;
    image.size = NSMakeSize(18.0, 18.0);
    button.image = image;
    button.toolTip = @"ClipType";

    NSMenu *menu = [[NSMenu alloc] initWithTitle:@"ClipType"];
    [self addItem:@"Trigger now" tag:CT_COMMAND_TRIGGER to:menu];
    [self addItem:@"Cancel active session" tag:CT_COMMAND_CANCEL to:menu];
    [menu addItem:[NSMenuItem separatorItem]];
    [self addItem:@"Open Settings…" tag:CT_COMMAND_OPEN_SETTINGS to:menu];
    _enabledItem = [self addItem:@"Enabled" tag:CT_COMMAND_TOGGLE_ENABLED to:menu];
    _modeItem = [[NSMenuItem alloc] initWithTitle:@"Mode: Auto" action:nil keyEquivalent:@""];
    _modeItem.enabled = NO;
    [menu addItem:_modeItem];
    _permissionItem = [self addItem:@"Accessibility: Not granted"
                                  tag:CT_COMMAND_PERMISSION to:menu];
    _startupItem = [self addItem:@"Start at Login" tag:CT_COMMAND_TOGGLE_STARTUP to:menu];
    [menu addItem:[NSMenuItem separatorItem]];
    [self addItem:@"About ClipType" tag:CT_COMMAND_ABOUT to:menu];
    [self addItem:@"Quit ClipType" tag:CT_COMMAND_QUIT to:menu];
    _statusItem.menu = menu;
    return self;
}

- (NSMenuItem *)addItem:(NSString *)title tag:(NSInteger)tag to:(NSMenu *)menu {
    NSMenuItem *item = [[NSMenuItem alloc] initWithTitle:title
                                                  action:@selector(handleCommand:)
                                           keyEquivalent:@""];
    item.target = self;
    item.tag = tag;
    [menu addItem:item];
    return item;
}

- (void)handleCommand:(NSMenuItem *)sender {
    if (_callback != NULL) _callback((int)sender.tag, _context);
}

- (void)updateEnabled:(BOOL)enabled mode:(int)mode permission:(int)permission startup:(BOOL)startup {
    _enabledItem.state = enabled ? NSControlStateValueOn : NSControlStateValueOff;
    NSArray<NSString *> *modes = @[ @"Keyboard", @"Clipboard", @"Auto", @"Code" ];
    int safeMode = (mode >= 0 && mode < 4) ? mode : 2;
    _modeItem.title = [NSString stringWithFormat:@"Mode: %@", modes[(NSUInteger)safeMode]];
    NSArray<NSString *> *permissions = @[
        @"Not required", @"Not requested", @"Not granted",
        @"Granted", @"Revoked", @"Unknown"
    ];
    int safePermission = (permission >= 0 && permission < 6) ? permission : 5;
    _permissionItem.title = [NSString stringWithFormat:@"Accessibility: %@",
                             permissions[(NSUInteger)safePermission]];
    _startupItem.state = startup ? NSControlStateValueOn : NSControlStateValueOff;
}
@end

void *ct_macos_status_create(CTCommandCallback callback, void *context) {
    @autoreleasepool {
        CTStatusController *controller = [[CTStatusController alloc]
            initWithCallback:callback context:context];
        return (__bridge_retained void *)controller;
    }
}

void ct_macos_status_update(
    void *pointer,
    int enabled,
    int mode,
    int permission,
    int startup
) {
    @autoreleasepool {
        CTStatusController *controller = (__bridge CTStatusController *)pointer;
        [controller updateEnabled:enabled != 0
                             mode:mode
                       permission:permission
                          startup:startup != 0];
    }
}

void ct_macos_status_destroy(void *pointer) {
    if (pointer == NULL) return;
    @autoreleasepool {
        CTStatusController *controller = (__bridge_transfer CTStatusController *)pointer;
        if (controller.statusItem != nil) {
            [[NSStatusBar systemStatusBar] removeStatusItem:controller.statusItem];
        }
    }
}

int ct_macos_startup_status(void) {
    if (@available(macOS 13.0, *)) {
        switch ([SMAppService mainAppService].status) {
            case SMAppServiceStatusNotRegistered: return CT_STARTUP_NOT_REGISTERED;
            case SMAppServiceStatusEnabled: return CT_STARTUP_ENABLED;
            case SMAppServiceStatusRequiresApproval: return CT_STARTUP_REQUIRES_APPROVAL;
            case SMAppServiceStatusNotFound: return CT_STARTUP_NOT_FOUND;
            default: return CT_STARTUP_UNKNOWN;
        }
    }
    return CT_STARTUP_UNSUPPORTED;
}

int ct_macos_set_startup(int enabled) {
    if (@available(macOS 13.0, *)) {
        NSError *error = nil;
        BOOL ok = enabled
            ? [[SMAppService mainAppService] registerAndReturnError:&error]
            : [[SMAppService mainAppService] unregisterAndReturnError:&error];
        return ok ? 1 : 0;
    }
    return 0;
}
