import Carbon
import Foundation

enum HotkeyAction {
    case trigger
    case cancel
}

enum HotkeyAvailability: String {
    case available
    case conflict
    case reserved
    case unsupported
    case unknown
}

struct NativeShortcut: Equatable {
    let keyCode: UInt32
    let modifiers: UInt32
    let canonical: String
}

/// Owns the application's two Carbon registrations on the main run loop.
/// Candidates are registered before the old pair is released, so a failed
/// replacement leaves the old pair untouched.
final class HotkeyManager {
    typealias CommandHandler = (HotkeyAction) -> Void

    private let onAction: CommandHandler
    private var eventHandler: EventHandlerRef?
    private var triggerRef: EventHotKeyRef?
    private var cancelRef: EventHotKeyRef?
    private var triggerID: UInt32 = 0
    private var cancelID: UInt32 = 0
    private var nextID: UInt32 = 100
    private(set) var currentTrigger: NativeShortcut?
    private(set) var currentCancel: NativeShortcut?

    init(trigger: String, cancel: String, onAction: @escaping CommandHandler) {
        self.onAction = onAction
        installEventHandler()
        _ = replacePair(trigger: trigger, cancel: cancel)
    }

    deinit {
        if let triggerRef { UnregisterEventHotKey(triggerRef) }
        if let cancelRef { UnregisterEventHotKey(cancelRef) }
        if let eventHandler { RemoveEventHandler(eventHandler) }
    }

    var currentPair: (trigger: String, cancel: String)? {
        guard let currentTrigger, let currentCancel else { return nil }
        return (currentTrigger.canonical, currentCancel.canonical)
    }

    func probePair(trigger: String, cancel: String) -> HotkeyAvailability {
        guard let candidate = parsePair(trigger: trigger, cancel: cancel) else {
            return .unsupported
        }
        if candidate.0 == currentTrigger && candidate.1 == currentCancel {
            return .available
        }

        var temporaryTrigger: EventHotKeyRef?
        var temporaryCancel: EventHotKeyRef?
        let triggerSame = candidate.0 == currentTrigger
        let cancelSame = candidate.1 == currentCancel
        if !triggerSame {
            let result = register(candidate.0, id: nextIdentifier())
            guard case .success(let reference) = result else {
                return result.failureValue
            }
            temporaryTrigger = reference
        }
        if !cancelSame {
            let result = register(candidate.1, id: nextIdentifier())
            guard case .success(let reference) = result else {
                if let temporaryTrigger { UnregisterEventHotKey(temporaryTrigger) }
                return result.failureValue
            }
            temporaryCancel = reference
        }
        if let temporaryTrigger { UnregisterEventHotKey(temporaryTrigger) }
        if let temporaryCancel { UnregisterEventHotKey(temporaryCancel) }
        return .available
    }

    @discardableResult
    func replacePair(trigger: String, cancel: String) -> HotkeyAvailability {
        guard let candidate = parsePair(trigger: trigger, cancel: cancel) else {
            return .unsupported
        }
        if candidate.0 == currentTrigger && candidate.1 == currentCancel {
            return .available
        }

        let triggerSame = candidate.0 == currentTrigger
        let cancelSame = candidate.1 == currentCancel
        var newTrigger: EventHotKeyRef?
        var newCancel: EventHotKeyRef?
        var newTriggerID = triggerID
        var newCancelID = cancelID

        if !triggerSame {
            newTriggerID = nextIdentifier()
            let result = register(candidate.0, id: newTriggerID)
            guard case .success(let reference) = result else {
                return result.failureValue
            }
            newTrigger = reference
        }
        if !cancelSame {
            newCancelID = nextIdentifier()
            let result = register(candidate.1, id: newCancelID)
            guard case .success(let reference) = result else {
                if let newTrigger { UnregisterEventHotKey(newTrigger) }
                return result.failureValue
            }
            newCancel = reference
        }

        // Both new registrations succeeded. Commit the pair only now.
        if !triggerSame, let triggerRef { UnregisterEventHotKey(triggerRef) }
        if !cancelSame, let cancelRef { UnregisterEventHotKey(cancelRef) }
        if !triggerSame {
            self.triggerRef = newTrigger
            triggerID = newTriggerID
            currentTrigger = candidate.0
        }
        if !cancelSame {
            self.cancelRef = newCancel
            cancelID = newCancelID
            currentCancel = candidate.1
        }
        return .available
    }

    private func installEventHandler() {
        var type = EventTypeSpec(
            eventClass: OSType(kEventClassKeyboard),
            eventKind: UInt32(kEventHotKeyPressed)
        )
        let context = Unmanaged.passUnretained(self).toOpaque()
        let status = InstallEventHandler(
            GetApplicationEventTarget(),
            { _, event, userData in
                guard let event, let userData else { return OSStatus(eventNotHandledErr) }
                let manager = Unmanaged<HotkeyManager>
                    .fromOpaque(userData)
                    .takeUnretainedValue()
                return manager.handle(event)
            },
            1,
            &type,
            context,
            &eventHandler
        )
        if status != noErr {
            eventHandler = nil
        }
    }

    private func handle(_ event: EventRef) -> OSStatus {
        var identifier = EventHotKeyID(signature: 0, id: 0)
        let status = withUnsafeMutablePointer(to: &identifier) { pointer in
            GetEventParameter(
                event,
                EventParamName(kEventParamDirectObject),
                DescType(typeEventHotKeyID),
                nil,
                MemoryLayout<EventHotKeyID>.size,
                nil,
                pointer
            )
        }
        guard status == noErr else { return status }
        if identifier.id == triggerID {
            onAction(.trigger)
            return noErr
        }
        if identifier.id == cancelID {
            onAction(.cancel)
            return noErr
        }
        return OSStatus(eventNotHandledErr)
    }

    private enum RegistrationResult {
        case success(EventHotKeyRef)
        case failure(HotkeyAvailability)

        var failureValue: HotkeyAvailability {
            guard case .failure(let value) = self else { return .unknown }
            return value
        }
    }

    private func register(
        _ shortcut: NativeShortcut,
        id: UInt32
    ) -> RegistrationResult {
        var reference: EventHotKeyRef?
        let identifier = EventHotKeyID(signature: 0x436C5470, id: id)
        let status = RegisterEventHotKey(
            shortcut.keyCode,
            shortcut.modifiers,
            identifier,
            GetApplicationEventTarget(),
            0,
            &reference
        )
        guard status == noErr, let reference else {
            if status == eventHotKeyExistsErr || status == eventHotKeyInvalidErr {
                return .failure(.conflict)
            }
            return .failure(.unknown)
        }
        return .success(reference)
    }

    private func nextIdentifier() -> UInt32 {
        defer { nextID &+= 1 }
        return nextID
    }

    private func parsePair(
        trigger: String,
        cancel: String
    ) -> (NativeShortcut, NativeShortcut)? {
        guard let trigger = parse(trigger), let cancel = parse(cancel) else {
            return nil
        }
        guard trigger != cancel else { return nil }
        return (trigger, cancel)
    }

    private func parse(_ value: String) -> NativeShortcut? {
        var modifiers: UInt32 = 0
        var key: String?
        for rawToken in value.split(separator: "+", omittingEmptySubsequences: false) {
            let token = rawToken.trimmingCharacters(in: .whitespacesAndNewlines).lowercased()
            guard !token.isEmpty else { return nil }
            switch token {
            case "ctrl", "control": modifiers |= UInt32(controlKey)
            case "alt", "option": modifiers |= UInt32(optionKey)
            case "shift": modifiers |= UInt32(shiftKey)
            case "meta", "cmd", "command": modifiers |= UInt32(cmdKey)
            default:
                guard key == nil else { return nil }
                key = token
            }
        }
        guard modifiers & UInt32(controlKey | optionKey | cmdKey) != 0,
              let key,
              let code = keyCodes[key]
        else { return nil }
        return NativeShortcut(
            keyCode: code,
            modifiers: modifiers,
            canonical: canonical(modifiers: modifiers, key: key)
        )
    }

    private func canonical(modifiers: UInt32, key: String) -> String {
        var values: [String] = []
        if modifiers & UInt32(controlKey) != 0 { values.append("ctrl") }
        if modifiers & UInt32(optionKey) != 0 { values.append("alt") }
        if modifiers & UInt32(shiftKey) != 0 { values.append("shift") }
        if modifiers & UInt32(cmdKey) != 0 { values.append("meta") }
        values.append(key)
        return values.joined(separator: "+")
    }

    private let keyCodes: [String: UInt32] = [
        "a": 0, "s": 1, "d": 2, "f": 3, "h": 4, "g": 5,
        "z": 6, "x": 7, "c": 8, "v": 9, "b": 11, "q": 12,
        "w": 13, "e": 14, "r": 15, "y": 16, "t": 17,
        "1": 18, "2": 19, "3": 20, "4": 21, "6": 22, "5": 23,
        "equal": 24, "9": 25, "7": 26, "minus": 27, "8": 28,
        "0": 29, "bracket-right": 30, "o": 31, "u": 32,
        "bracket-left": 33, "i": 34, "p": 35, "enter": 36,
        "l": 37, "j": 38, "quote": 39, "k": 40, "semicolon": 41,
        "backslash": 42, "comma": 43, "slash": 44, "n": 45,
        "m": 46, "period": 47, "tab": 48, "space": 49,
        "backquote": 50, "backspace": 51, "escape": 53,
        "f17": 64, "f18": 79, "f19": 80, "f20": 90, "f5": 96,
        "f6": 97, "f7": 98, "f3": 99, "f8": 100, "f9": 101,
        "f11": 103, "f13": 105, "f16": 106, "f14": 107,
        "f10": 109, "f12": 111, "f15": 113, "insert": 114,
        "home": 115, "pageup": 116, "delete": 117, "f4": 118,
        "end": 119, "f2": 120, "pagedown": 121, "f1": 122,
        "left": 123, "right": 124, "down": 125, "up": 126,
    ]
}
