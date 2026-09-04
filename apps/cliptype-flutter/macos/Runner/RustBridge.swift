import Foundation

struct RustSnapshot {
    let enabled: Bool
    let notifications: Bool
    let startAtLogin: Bool
    let mode: Int
    let charactersPerSecond: Int
    let jitterPercent: Int
    let typoProbabilityPercent: Int
    let autoClipboardThreshold: Int
    let generation: UInt64
    let phase: Int
    let backend: Int
    let completion: Int
    let batchesCompleted: Int

    var phaseName: String {
        switch phase {
        case CT_BRIDGE_PHASE_PREPARING: return "preparing"
        case CT_BRIDGE_PHASE_INJECTING: return "injecting"
        case CT_BRIDGE_PHASE_CANCELLING: return "cancelling"
        default: return "idle"
        }
    }

    var backendName: String? {
        switch backend {
        case CT_BRIDGE_BACKEND_KEYBOARD: return "keyboard"
        case CT_BRIDGE_BACKEND_CLIPBOARD: return "clipboard"
        case CT_BRIDGE_BACKEND_CODE: return "code"
        default: return nil
        }
    }

    var completionName: String? {
        switch completion {
        case CT_BRIDGE_COMPLETION_COMPLETED: return "completed"
        case CT_BRIDGE_COMPLETION_CANCELLED: return "cancelled"
        case CT_BRIDGE_COMPLETION_TARGET_CHANGED: return "target_changed"
        case CT_BRIDGE_COMPLETION_CLIPBOARD_CHANGED: return "clipboard_changed"
        case CT_BRIDGE_COMPLETION_PERMISSION: return "permission"
        case CT_BRIDGE_COMPLETION_FAILED: return "failed"
        default: return nil
        }
    }
}

final class RustBridge {
    private(set) var handle: CTBridgeHandle?

    init() {
        handle = ct_bridge_create()
    }

    var isAvailable: Bool { handle != nil }

    deinit {
        if let handle {
            ct_bridge_destroy(handle)
        }
    }

    func snapshot() -> RustSnapshot? {
        guard let handle else { return nil }
        var state = CTBridgeState(
            enabled: 0,
            notifications: 0,
            start_at_login: 0,
            mode: Int32(CT_BRIDGE_MODE_AUTO),
            characters_per_second: 0,
            jitter_percent: 0,
            typo_probability_percent: 0,
            auto_clipboard_threshold: 0,
            generation: 0,
            phase: Int32(CT_BRIDGE_PHASE_IDLE),
            backend: Int32(CT_BRIDGE_BACKEND_NONE),
            completion: Int32(CT_BRIDGE_COMPLETION_NONE),
            batches_completed: 0
        )
        let status = withUnsafeMutablePointer(to: &state) { pointer in
            ct_bridge_get_state(handle, pointer)
        }
        guard Int(status) == CT_BRIDGE_OK else { return nil }
        return RustSnapshot(
            enabled: state.enabled != 0,
            notifications: state.notifications != 0,
            startAtLogin: state.start_at_login != 0,
            mode: Int(state.mode),
            charactersPerSecond: Int(state.characters_per_second),
            jitterPercent: Int(state.jitter_percent),
            typoProbabilityPercent: Int(state.typo_probability_percent),
            autoClipboardThreshold: Int(state.auto_clipboard_threshold),
            generation: state.generation,
            phase: Int(state.phase),
            backend: Int(state.backend),
            completion: Int(state.completion),
            batchesCompleted: Int(state.batches_completed)
        )
    }

    func hotkey(trigger: Bool) -> String? {
        guard let handle else { return nil }
        var bytes = [CChar](repeating: 0, count: 64)
        let status = bytes.withUnsafeMutableBufferPointer { buffer in
            ct_bridge_get_hotkey(
                handle,
                trigger ? 1 : 0,
                buffer.baseAddress,
                buffer.count
            )
        }
        guard Int(status) == CT_BRIDGE_OK else { return nil }
        return String(cString: bytes)
    }

    func validateHotkeys(trigger: String, cancel: String) -> Int {
        trigger.withCString { triggerPointer in
            cancel.withCString { cancelPointer in
                Int(ct_bridge_validate_hotkeys(triggerPointer, cancelPointer))
            }
        }
    }

    func saveSettings(
        enabled: Bool,
        notifications: Bool,
        startAtLogin: Bool,
        mode: Int32,
        charactersPerSecond: Int,
        jitterPercent: Int,
        typoProbabilityPercent: Int,
        autoClipboardThreshold: Int,
        trigger: String,
        cancel: String
    ) -> Int {
        guard let handle else { return CT_BRIDGE_NATIVE_FAILURE }
        return trigger.withCString { triggerPointer in
            cancel.withCString { cancelPointer in
                Int(ct_bridge_save_settings(
                    handle,
                    enabled ? 1 : 0,
                    notifications ? 1 : 0,
                    startAtLogin ? 1 : 0,
                    mode,
                    UInt16(clamping: charactersPerSecond),
                    UInt8(clamping: jitterPercent),
                    UInt8(clamping: typoProbabilityPercent),
                    UInt32(clamping: autoClipboardThreshold),
                    triggerPointer,
                    cancelPointer
                ))
            }
        }
    }

    func trigger() -> Int {
        guard let handle else { return CT_BRIDGE_NATIVE_FAILURE }
        return Int(ct_bridge_trigger(handle))
    }

    func cancel() -> Int {
        guard let handle else { return CT_BRIDGE_NATIVE_FAILURE }
        return Int(ct_bridge_cancel(handle))
    }

    func shutdown() -> Int {
        guard let handle else { return CT_BRIDGE_OK }
        return Int(ct_bridge_shutdown(handle))
    }
}
