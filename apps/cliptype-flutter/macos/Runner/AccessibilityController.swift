import ApplicationServices
import Cocoa

final class AccessibilityController {
    private var requestWasMade = false
    private var wasGranted = false
    private var observationTimer: Timer?
    private var observationDeadline: Date?

    var state: String {
        if AXIsProcessTrusted() {
            wasGranted = true
            return "granted"
        }
        if !requestWasMade {
            return "not_requested"
        }
        return wasGranted ? "revoked" : "not_granted"
    }

    func request() -> String {
        if state == "granted" { return "already_granted" }
        requestWasMade = true
        let options = [kAXTrustedCheckOptionPrompt.takeUnretainedValue() as String: true] as CFDictionary
        _ = AXIsProcessTrustedWithOptions(options)
        return "prompt_requested"
    }

    func openSystemSettings() -> String {
        let urls = [
            "x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_Accessibility",
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
        ].compactMap(URL.init(string:))
        for url in urls where NSWorkspace.shared.open(url) {
            return "settings_opened"
        }
        return "native_failure"
    }

    func observePermissionChanges(onChange: @escaping (String) -> Void) {
        observationTimer?.invalidate()
        observationDeadline = Date().addingTimeInterval(12)
        observationTimer = Timer.scheduledTimer(withTimeInterval: 0.4, repeats: true) {
            [weak self] timer in
            guard let self else {
                timer.invalidate()
                return
            }
            let current = self.state
            onChange(current)
            if current == "granted" || Date() >= (self.observationDeadline ?? .distantPast) {
                timer.invalidate()
                self.observationTimer = nil
                self.observationDeadline = nil
            }
        }
    }

    func stopObservation() {
        observationTimer?.invalidate()
        observationTimer = nil
        observationDeadline = nil
    }
}
