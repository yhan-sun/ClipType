#ifndef CLIPTYPE_BRIDGE_H
#define CLIPTYPE_BRIDGE_H

#include <stdint.h>
#include <stddef.h>

typedef void *CTBridgeHandle;

enum {
    CT_BRIDGE_OK = 0,
    CT_BRIDGE_INVALID = 1,
    CT_BRIDGE_NATIVE_FAILURE = 2,
    CT_BRIDGE_BUSY = 3,
    CT_BRIDGE_SHUTTING_DOWN = 4,
    CT_BRIDGE_REJECTED = 5,
};

enum {
    CT_BRIDGE_HOTKEY_AVAILABLE = 0,
    CT_BRIDGE_HOTKEY_CONFLICT = 1,
    CT_BRIDGE_HOTKEY_RESERVED = 2,
    CT_BRIDGE_HOTKEY_UNSUPPORTED = 3,
    CT_BRIDGE_HOTKEY_UNKNOWN = 4,
};

enum {
    CT_BRIDGE_MODE_KEYBOARD = 0,
    CT_BRIDGE_MODE_CLIPBOARD = 1,
    CT_BRIDGE_MODE_AUTO = 2,
    CT_BRIDGE_MODE_CODE = 3,
};

enum {
    CT_BRIDGE_PHASE_IDLE = 0,
    CT_BRIDGE_PHASE_PREPARING = 1,
    CT_BRIDGE_PHASE_INJECTING = 2,
    CT_BRIDGE_PHASE_CANCELLING = 3,
};

enum {
    CT_BRIDGE_BACKEND_KEYBOARD = 0,
    CT_BRIDGE_BACKEND_CLIPBOARD = 1,
    CT_BRIDGE_BACKEND_CODE = 2,
    CT_BRIDGE_BACKEND_NONE = -1,
};

enum {
    CT_BRIDGE_COMPLETION_NONE = 0,
    CT_BRIDGE_COMPLETION_COMPLETED = 1,
    CT_BRIDGE_COMPLETION_CANCELLED = 2,
    CT_BRIDGE_COMPLETION_TARGET_CHANGED = 3,
    CT_BRIDGE_COMPLETION_CLIPBOARD_CHANGED = 4,
    CT_BRIDGE_COMPLETION_PERMISSION = 5,
    CT_BRIDGE_COMPLETION_FAILED = 6,
    CT_BRIDGE_COMPLETION_MODIFIER_CONFLICT = 7,
    CT_BRIDGE_COMPLETION_TARGET_EVIDENCE_UNAVAILABLE = 8,
    CT_BRIDGE_COMPLETION_TARGET_DISAPPEARED = 9,
    CT_BRIDGE_COMPLETION_PARTIAL_INPUT = 10,
    CT_BRIDGE_COMPLETION_PROGRESS_UNKNOWN = 11,
    CT_BRIDGE_COMPLETION_BLOCKED_CAUSE_UNKNOWN = 12,
    CT_BRIDGE_COMPLETION_NATIVE_FAILURE = 13,
    CT_BRIDGE_COMPLETION_INTERNAL_INVARIANT = 14,
    CT_BRIDGE_COMPLETION_MODIFIER_TIMEOUT = 15,
};

typedef struct {
    int32_t enabled;
    int32_t notifications;
    int32_t start_at_login;
    int32_t mode;
    uint16_t characters_per_second;
    uint8_t jitter_percent;
    uint8_t typo_probability_percent;
    uint32_t auto_clipboard_threshold;
    uint64_t generation;
    int32_t phase;
    int32_t backend;
    int32_t completion;
    uint32_t batches_completed;
} CTBridgeState;

/*
 * The handle is created and destroyed by the main thread. Calls are
 * synchronous and must not overlap. `ct_bridge_trigger` may start the bounded
 * Rust worker; no clipboard or input plaintext crosses this ABI.
 */
CTBridgeHandle ct_bridge_create(void);
void ct_bridge_destroy(CTBridgeHandle handle);
int32_t ct_bridge_get_state(CTBridgeHandle handle, CTBridgeState *output);
int32_t ct_bridge_get_hotkey(CTBridgeHandle handle, int32_t trigger,
                             char *output, size_t capacity);
int32_t ct_bridge_validate_hotkeys(const char *trigger, const char *cancel);
int32_t ct_bridge_save_settings(CTBridgeHandle handle,
                                int32_t enabled,
                                int32_t notifications,
                                int32_t start_at_login,
                                int32_t mode,
                                uint16_t characters_per_second,
                                uint8_t jitter_percent,
                                uint8_t typo_probability_percent,
                                uint32_t auto_clipboard_threshold,
                                const char *trigger,
                                const char *cancel);
int32_t ct_bridge_trigger(CTBridgeHandle handle);
int32_t ct_bridge_cancel(CTBridgeHandle handle);
int32_t ct_bridge_shutdown(CTBridgeHandle handle);

#endif
