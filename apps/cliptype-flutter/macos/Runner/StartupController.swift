import ServiceManagement

final class StartupController {
    var state: String {
        guard #available(macOS 13.0, *) else { return "unsupported" }
        switch SMAppService.mainApp.status {
        case .notRegistered: return "not_registered"
        case .enabled: return "enabled"
        case .requiresApproval: return "requires_approval"
        case .notFound: return "not_found"
        @unknown default: return "unknown"
        }
    }

    func setEnabled(_ enabled: Bool) -> String {
        guard #available(macOS 13.0, *) else { return "unsupported" }
        do {
            if enabled {
                try SMAppService.mainApp.register()
            } else {
                try SMAppService.mainApp.unregister()
            }
            return "ok"
        } catch {
            return "native_failure"
        }
    }
}
