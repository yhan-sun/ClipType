import Cocoa

enum NativeCommand {
    case trigger
    case cancel
    case openSettings
    case toggleEnabled
    case toggleStartup
    case permission
    case about
    case quit
}

final class StatusItemController {
    private let onCommand: (NativeCommand) -> Void
    private let statusItem: NSStatusItem
    private var language: String
    private var triggerItem: NSMenuItem!
    private var cancelItem: NSMenuItem!
    private var settingsItem: NSMenuItem!
    private var enabledItem: NSMenuItem!
    private var modeItem: NSMenuItem!
    private var permissionItem: NSMenuItem!
    private var startupItem: NSMenuItem!
    private var aboutItem: NSMenuItem!
    private var quitItem: NSMenuItem!

    init(language: String = "en", onCommand: @escaping (NativeCommand) -> Void) {
        self.onCommand = onCommand
        self.language = language == "zh" ? "zh" : "en"
        statusItem = NSStatusBar.system.statusItem(withLength: NSStatusItem.squareLength)
        let button = statusItem.button
        let image = NSImage(
            systemSymbolName: "doc.on.clipboard",
            accessibilityDescription: "ClipType"
        )
        image?.isTemplate = true
        button?.image = image
        button?.toolTip = "ClipType"

        let menu = NSMenu(title: "ClipType")
        triggerItem = add("", command: .trigger, to: menu)
        cancelItem = add("", command: .cancel, to: menu)
        menu.addItem(.separator())
        settingsItem = add("", command: .openSettings, to: menu)
        enabledItem = add("", command: .toggleEnabled, to: menu)
        modeItem = NSMenuItem(title: "", action: nil, keyEquivalent: "")
        modeItem.isEnabled = false
        menu.addItem(modeItem)
        permissionItem = add("", command: .permission, to: menu)
        startupItem = add("", command: .toggleStartup, to: menu)
        menu.addItem(.separator())
        aboutItem = add("", command: .about, to: menu)
        quitItem = add("", command: .quit, to: menu)
        statusItem.menu = menu
        setLanguage(self.language)
    }

    var interfaceLanguage: String { language }

    func setLanguage(_ language: String) {
        self.language = language == "zh" ? "zh" : "en"
        triggerItem.title = text(english: "Trigger now", chinese: "立即触发")
        cancelItem.title = text(english: "Cancel active session", chinese: "取消活动会话")
        settingsItem.title = text(english: "Open Settings…", chinese: "打开设置…")
        enabledItem.title = text(english: "Enabled", chinese: "启用")
        startupItem.title = text(english: "Start at Login", chinese: "登录时启动")
        aboutItem.title = text(english: "About ClipType", chinese: "关于 ClipType")
        quitItem.title = text(english: "Quit ClipType", chinese: "退出 ClipType")
    }

    func update(snapshot: RustSnapshot?, permission: String, startup: String) {
        enabledItem.state = snapshot?.enabled == true ? .on : .off
        let mode = snapshot.map { modeName($0.mode) } ?? text(english: "Unavailable", chinese: "不可用")
        modeItem.title = "\(text(english: "Mode", chinese: "模式")): \(mode)"
        permissionItem.title = "\(text(english: "Accessibility", chinese: "辅助功能")): \(permissionName(permission))"
        startupItem.state = startup == "enabled" || startup == "requires_approval" ? .on : .off
    }

    private func add(
        _ title: String,
        command: NativeCommand,
        to menu: NSMenu
    ) -> NSMenuItem {
        let item = NSMenuItem(title: title, action: #selector(handleCommand(_:)), keyEquivalent: "")
        item.target = self
        item.tag = commandTag(command)
        menu.addItem(item)
        return item
    }

    @objc private func handleCommand(_ sender: NSMenuItem) {
        guard let command = commandFromTag(sender.tag) else { return }
        onCommand(command)
    }

    private func commandTag(_ command: NativeCommand) -> Int {
        switch command {
        case .trigger: return 1
        case .cancel: return 2
        case .openSettings: return 3
        case .toggleEnabled: return 4
        case .toggleStartup: return 5
        case .permission: return 6
        case .about: return 7
        case .quit: return 8
        }
    }

    private func commandFromTag(_ tag: Int) -> NativeCommand? {
        switch tag {
        case 1: return .trigger
        case 2: return .cancel
        case 3: return .openSettings
        case 4: return .toggleEnabled
        case 5: return .toggleStartup
        case 6: return .permission
        case 7: return .about
        case 8: return .quit
        default: return nil
        }
    }

    private func modeName(_ mode: Int) -> String {
        switch mode {
        case CT_BRIDGE_MODE_KEYBOARD: return text(english: "Keyboard", chinese: "键盘")
        case CT_BRIDGE_MODE_CLIPBOARD: return text(english: "Clipboard", chinese: "剪贴板")
        case CT_BRIDGE_MODE_CODE: return text(english: "Code", chinese: "代码")
        default: return text(english: "Auto", chinese: "自动")
        }
    }

    private func permissionName(_ state: String) -> String {
        switch state {
        case "granted": return text(english: "Granted", chinese: "已授权")
        case "revoked": return text(english: "Revoked", chinese: "已撤销")
        case "not_requested": return text(english: "Not requested", chinese: "未请求")
        case "not_granted": return text(english: "Not granted", chinese: "未授权")
        default: return text(english: "Unknown", chinese: "未知")
        }
    }

    private func text(english: String, chinese: String) -> String {
        language == "zh" ? chinese : english
    }
}
