import Cocoa
import FlutterMacOS

final class ClipTypeNativePlugin: NSObject, FlutterStreamHandler {
    private let languageDefaultsKey = "ClipType.interfaceLanguage"
    private weak var appDelegate: AppDelegate?
    private let rust: RustBridge
    private let accessibility = AccessibilityController()
    private let startup = StartupController()
    private var hotkeys: HotkeyManager!
    private var statusItem: StatusItemController!
    private var eventSink: FlutterEventSink?
    private var sessionTimer: Timer?
    private var methodChannel: FlutterMethodChannel?
    private var eventChannel: FlutterEventChannel?
    private var isShutdown = false

    init(appDelegate: AppDelegate) {
        self.appDelegate = appDelegate
        rust = RustBridge()
        super.init()

        let trigger = rust.hotkey(trigger: true) ?? "ctrl+alt+shift+v"
        let cancel = rust.hotkey(trigger: false) ?? "ctrl+alt+shift+x"
        hotkeys = HotkeyManager(trigger: trigger, cancel: cancel) { [weak self] action in
            switch action {
            case .trigger: self?.triggerFromNative()
            case .cancel: self?.cancelFromNative()
            }
        }
        let language = UserDefaults.standard.string(forKey: languageDefaultsKey) == "zh" ? "zh" : "en"
        statusItem = StatusItemController(language: language) { [weak self] command in
            self?.handle(command: command)
        }
        updateStatusItem()
    }

    func attach(to viewController: FlutterViewController) {
        guard methodChannel == nil else { return }
        let messenger = viewController.engine.binaryMessenger
        let methods = FlutterMethodChannel(name: "io.cliptype/native", binaryMessenger: messenger)
        let events = FlutterEventChannel(name: "io.cliptype/events", binaryMessenger: messenger)
        methods.setMethodCallHandler { [weak self] call, result in
            self?.handle(call: call, result: result)
        }
        events.setStreamHandler(self)
        methodChannel = methods
        eventChannel = events
        DispatchQueue.main.async { [weak self] in
            self?.send(type: "ready")
        }
    }

    func showSettings() {
        appDelegate?.showSettings()
    }

    func handle(command: NativeCommand) {
        switch command {
        case .trigger: triggerFromNative()
        case .cancel: cancelFromNative()
        case .openSettings, .about: showSettings()
        case .toggleEnabled: toggleEnabled()
        case .toggleStartup: toggleStartup()
        case .permission: _ = requestAccessibility()
        case .quit: appDelegate?.terminateClipType()
        }
    }

    func shutdown() {
        guard !isShutdown else { return }
        isShutdown = true
        sessionTimer?.invalidate()
        sessionTimer = nil
        accessibility.stopObservation()
        _ = rust.shutdown()
    }

    func onListen(
        withArguments arguments: Any?,
        eventSink events: @escaping FlutterEventSink
    ) -> FlutterError? {
        eventSink = events
        return nil
    }

    func onCancel(withArguments arguments: Any?) -> FlutterError? {
        eventSink = nil
        return nil
    }

    private func handle(call: FlutterMethodCall, result: @escaping FlutterResult) {
        switch call.method {
        case "getState":
            result(stateMap())
        case "getInterfaceLanguage":
            result(["language": statusItem.interfaceLanguage])
        case "setInterfaceLanguage":
            let args = call.arguments as? [String: Any] ?? [:]
            result(setInterfaceLanguage(args["language"] as? String))
        case "saveSettings":
            result(saveSettings(arguments: call.arguments as? [String: Any] ?? [:]))
        case "probeHotkeys":
            let args = call.arguments as? [String: Any] ?? [:]
            result(probeHotkeys(arguments: args))
        case "applyHotkeys":
            let args = call.arguments as? [String: Any] ?? [:]
            result(applyHotkeys(arguments: args))
        case "trigger":
            triggerFromFlutter(result)
        case "cancel":
            result(performCancel())
        case "requestAccessibility":
            result(requestAccessibility())
        case "openAccessibilitySettings":
            result(openAccessibilitySettings())
        case "setStartAtLogin":
            let args = call.arguments as? [String: Any] ?? [:]
            result(setStartAtLogin(enabled: boolArg(args["enabled"])))
        case "quit":
            appDelegate?.terminateClipType()
            result(["result": "ok"])
        default:
            result(FlutterMethodNotImplemented)
        }
    }

    private func stateMap() -> [String: Any] {
        let snapshot = rust.snapshot()
        let mode: String
        if let snapshot {
            switch snapshot.mode {
            case CT_BRIDGE_MODE_KEYBOARD: mode = "keyboard"
            case CT_BRIDGE_MODE_CLIPBOARD: mode = "clipboard"
            case CT_BRIDGE_MODE_CODE: mode = "code"
            default: mode = "auto"
            }
        } else {
            mode = "auto"
        }
        return [
            "enabled": snapshot?.enabled ?? true,
            "notifications": snapshot?.notifications ?? true,
            "startAtLogin": snapshot?.startAtLogin ?? false,
            "mode": mode,
            "charactersPerSecond": snapshot?.charactersPerSecond ?? 40,
            "jitterPercent": snapshot?.jitterPercent ?? 0,
            "typoProbabilityPercent": snapshot?.typoProbabilityPercent ?? 0,
            "autoClipboardThreshold": snapshot?.autoClipboardThreshold ?? 256,
            "triggerHotkey": rust.hotkey(trigger: true) ?? "ctrl+alt+shift+v",
            "cancelHotkey": rust.hotkey(trigger: false) ?? "ctrl+alt+shift+x",
            "phase": snapshot?.phaseName ?? "idle",
            "backend": snapshot?.backendName ?? NSNull(),
            "completion": snapshot?.completionName ?? NSNull(),
            "generation": snapshot?.generation ?? 0,
            "batchesCompleted": snapshot?.batchesCompleted ?? 0,
            "permission": accessibility.state,
            "startup": startup.state,
            "bridgeError": rust.isAvailable ? NSNull() : "bridge_unavailable",
        ]
    }

    private func setInterfaceLanguage(_ language: String?) -> [String: Any] {
        guard let language, language == "en" || language == "zh" else {
            return ["result": "invalid"]
        }
        UserDefaults.standard.set(language, forKey: languageDefaultsKey)
        statusItem.setLanguage(language)
        updateStatusItem()
        return ["result": "ok", "language": language]
    }

    private func saveSettings(arguments: [String: Any]) -> [String: Any] {
        guard let values = settingValues(arguments: arguments) else {
            return ["result": "invalid"]
        }
        guard rust.isAvailable else { return ["result": "native_failure"] }

        let validation = rust.validateHotkeys(trigger: values.trigger, cancel: values.cancel)
        guard validation == CT_BRIDGE_HOTKEY_AVAILABLE else {
            let result = availabilityResult(validation)
            send(type: "hotkeyConflict", extra: ["availability": result])
            return ["result": result]
        }

        let oldPair = hotkeys.currentPair
        let currentStart = rust.snapshot()?.startAtLogin ?? false
        let startupChanged = currentStart != values.startAtLogin
        if startupChanged {
            let startupResult = startup.setEnabled(values.startAtLogin)
            guard startupResult == "ok" else {
                return ["result": startupResult]
            }
        }

        let hotkeysChanged = oldPair?.trigger != values.trigger || oldPair?.cancel != values.cancel
        if hotkeysChanged {
            let availability = hotkeys.replacePair(trigger: values.trigger, cancel: values.cancel)
            guard availability == .available else {
                if startupChanged { _ = startup.setEnabled(currentStart) }
                send(type: "hotkeyConflict", extra: ["availability": availability.rawValue])
                return ["result": availability.rawValue]
            }
        }

        let status = rust.saveSettings(
            enabled: values.enabled,
            notifications: values.notifications,
            startAtLogin: values.startAtLogin,
            mode: values.mode,
            charactersPerSecond: values.charactersPerSecond,
            jitterPercent: values.jitterPercent,
            typoProbabilityPercent: values.typoProbabilityPercent,
            autoClipboardThreshold: values.autoClipboardThreshold,
            trigger: values.trigger,
            cancel: values.cancel
        )
        guard status == CT_BRIDGE_OK else {
            if hotkeysChanged, let oldPair {
                _ = hotkeys.replacePair(trigger: oldPair.trigger, cancel: oldPair.cancel)
            }
            if startupChanged { _ = startup.setEnabled(currentStart) }
            return ["result": bridgeResult(status)]
        }

        send(type: "settingsChanged")
        if hotkeysChanged { send(type: "hotkeysApplied") }
        updateStatusItem()
        return ["result": "ok"]
    }

    private func probeHotkeys(arguments: [String: Any]) -> [String: Any] {
        guard let trigger = arguments["triggerHotkey"] as? String,
              let cancel = arguments["cancelHotkey"] as? String
        else {
            return availabilityMap(.unsupported)
        }
        let validation = rust.validateHotkeys(trigger: trigger, cancel: cancel)
        guard validation == CT_BRIDGE_HOTKEY_AVAILABLE else {
            return availabilityMap(availabilityCode(validation))
        }
        return availabilityMap(hotkeys.probePair(trigger: trigger, cancel: cancel))
    }

    private func applyHotkeys(arguments: [String: Any]) -> [String: Any] {
        guard let trigger = arguments["triggerHotkey"] as? String,
              let cancel = arguments["cancelHotkey"] as? String,
              let current = rust.snapshot()
        else {
            return ["result": "invalid"]
        }
        var values = arguments
        values["enabled"] = current.enabled
        values["notifications"] = current.notifications
        values["startAtLogin"] = current.startAtLogin
        values["mode"] = modeName(current.mode)
        values["charactersPerSecond"] = current.charactersPerSecond
        values["jitterPercent"] = current.jitterPercent
        values["typoProbabilityPercent"] = current.typoProbabilityPercent
        values["autoClipboardThreshold"] = current.autoClipboardThreshold
        values["triggerHotkey"] = trigger
        values["cancelHotkey"] = cancel
        return saveSettings(arguments: values)
    }

    private func settingValues(arguments: [String: Any]) -> SettingValues? {
        guard let mode = modeValue(arguments["mode"]),
              let trigger = arguments["triggerHotkey"] as? String,
              let cancel = arguments["cancelHotkey"] as? String
        else { return nil }
        return SettingValues(
            enabled: boolArg(arguments["enabled"]),
            notifications: boolArg(arguments["notifications"]),
            startAtLogin: boolArg(arguments["startAtLogin"]),
            mode: mode,
            charactersPerSecond: intArg(arguments["charactersPerSecond"]),
            jitterPercent: intArg(arguments["jitterPercent"]),
            typoProbabilityPercent: intArg(arguments["typoProbabilityPercent"]),
            autoClipboardThreshold: intArg(arguments["autoClipboardThreshold"]),
            trigger: trigger,
            cancel: cancel
        )
    }

    private func setStartAtLogin(enabled: Bool) -> [String: Any] {
        var values = stateMap()
        values["startAtLogin"] = enabled
        return saveSettings(arguments: values)
    }

    private func toggleEnabled() {
        var values = stateMap()
        values["enabled"] = !(values["enabled"] as? Bool ?? true)
        _ = saveSettings(arguments: values)
    }

    private func toggleStartup() {
        var values = stateMap()
        values["startAtLogin"] = !(values["startAtLogin"] as? Bool ?? false)
        _ = saveSettings(arguments: values)
    }

    private func requestAccessibility() -> [String: Any] {
        let result = accessibility.request()
        send(type: "permissionChanged")
        updateStatusItem()
        if result == "prompt_requested" {
            observeAccessibilityChanges()
        }
        return ["result": result]
    }

    private func openAccessibilitySettings() -> [String: Any] {
        let result = accessibility.openSystemSettings()
        if result == "settings_opened" {
            observeAccessibilityChanges()
        }
        return ["result": result]
    }

    private func observeAccessibilityChanges() {
        accessibility.observePermissionChanges { [weak self] _ in
            self?.send(type: "permissionChanged")
            self?.updateStatusItem()
        }
    }

    private func triggerFromNative() {
        _ = performTrigger()
    }

    private func triggerFromFlutter(_ result: @escaping FlutterResult) {
        guard accessibility.state == "granted" else {
            result(performTrigger())
            return
        }
        guard appDelegate?.prepareForExternalTrigger() == true else {
            result(performTrigger())
            return
        }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) { [weak self] in
            result(self?.performTrigger() ?? ["result": "native_failure"])
        }
    }

    private func cancelFromNative() {
        _ = performCancel()
    }

    private func performTrigger() -> [String: Any] {
        guard accessibility.state == "granted" else {
            updateStatusItem()
            return ["result": "permission_required"]
        }
        let result = rust.trigger()
        let mapped = bridgeResult(result, trigger: true)
        if result == CT_BRIDGE_OK {
            send(type: "sessionStarted")
            observeSession()
        }
        updateStatusItem()
        return ["result": mapped]
    }

    private func performCancel() -> [String: Any] {
        let result = rust.cancel()
        let mapped = result == CT_BRIDGE_OK ? "cancel_requested" :
            (result == CT_BRIDGE_REJECTED ? "idle" : bridgeResult(result))
        updateStatusItem()
        return ["result": mapped]
    }

    private func observeSession() {
        sessionTimer?.invalidate()
        sessionTimer = Timer.scheduledTimer(withTimeInterval: 0.12, repeats: true) {
            [weak self] _ in
            self?.pollSession()
        }
    }

    private func pollSession() {
        updateStatusItem()
        guard let snapshot = rust.snapshot(), snapshot.phase == CT_BRIDGE_PHASE_IDLE else {
            return
        }
        sessionTimer?.invalidate()
        sessionTimer = nil
        let completion = snapshot.completionName ?? "failed"
        let event: String
        if completion == "cancelled" {
            event = "sessionCancelled"
        } else if completion == "completed" {
            event = "sessionCompleted"
        } else {
            event = "sessionFailed"
        }
        send(type: event, extra: ["completion": completion])
        updateStatusItem()
    }

    private func updateStatusItem() {
        statusItem?.update(
            snapshot: rust.snapshot(),
            permission: accessibility.state,
            startup: startup.state
        )
    }

    private func send(type: String, extra: [String: Any] = [:]) {
        var payload = extra
        payload["type"] = type
        eventSink?(payload)
    }

    private func availabilityMap(_ value: HotkeyAvailability) -> [String: Any] {
        [
            "result": value.rawValue,
            "overall": value.rawValue,
            "trigger": value.rawValue,
            "cancel": value.rawValue,
        ]
    }

    private func availabilityCode(_ code: Int) -> HotkeyAvailability {
        switch code {
        case CT_BRIDGE_HOTKEY_CONFLICT: return .conflict
        case CT_BRIDGE_HOTKEY_RESERVED: return .reserved
        case CT_BRIDGE_HOTKEY_UNSUPPORTED, CT_BRIDGE_INVALID: return .unsupported
        default: return .unknown
        }
    }

    private func availabilityResult(_ code: Int) -> String {
        availabilityCode(code).rawValue
    }

    private func bridgeResult(_ code: Int, trigger: Bool = false) -> String {
        switch code {
        case CT_BRIDGE_OK: return trigger ? "started" : "ok"
        case CT_BRIDGE_BUSY: return "busy"
        case CT_BRIDGE_SHUTTING_DOWN: return "shutting_down"
        case CT_BRIDGE_REJECTED: return "rejected"
        case CT_BRIDGE_INVALID: return "invalid"
        default: return "native_failure"
        }
    }

    private func modeName(_ mode: Int) -> String {
        switch mode {
        case CT_BRIDGE_MODE_KEYBOARD: return "keyboard"
        case CT_BRIDGE_MODE_CLIPBOARD: return "clipboard"
        case CT_BRIDGE_MODE_CODE: return "code"
        default: return "auto"
        }
    }

    private func boolArg(_ value: Any?) -> Bool {
        if let value = value as? Bool { return value }
        if let value = value as? NSNumber { return value.boolValue }
        return false
    }

    private func intArg(_ value: Any?) -> Int {
        if let value = value as? Int { return value }
        if let value = value as? NSNumber { return value.intValue }
        return 0
    }

    private func modeValue(_ value: Any?) -> Int32? {
        guard let value = value as? String else { return nil }
        switch value {
        case "keyboard": return Int32(CT_BRIDGE_MODE_KEYBOARD)
        case "clipboard": return Int32(CT_BRIDGE_MODE_CLIPBOARD)
        case "auto": return Int32(CT_BRIDGE_MODE_AUTO)
        case "code": return Int32(CT_BRIDGE_MODE_CODE)
        default: return nil
        }
    }
}

private struct SettingValues {
    let enabled: Bool
    let notifications: Bool
    let startAtLogin: Bool
    let mode: Int32
    let charactersPerSecond: Int
    let jitterPercent: Int
    let typoProbabilityPercent: Int
    let autoClipboardThreshold: Int
    let trigger: String
    let cancel: String
}
