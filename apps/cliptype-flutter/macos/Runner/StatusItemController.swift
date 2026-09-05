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
    private var stateItem: NSMenuItem!
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
        let image = Self.makeTemplateStatusImage()
        statusItem.button?.image = image
        statusItem.button?.setAccessibilityLabel("ClipType")
        statusItem.button?.toolTip = "ClipType"

        let menu = NSMenu(title: "ClipType")
        stateItem = NSMenuItem(title: "", action: nil, keyEquivalent: "")
        stateItem.isEnabled = false
        menu.addItem(stateItem)
        menu.addItem(.separator())
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
        triggerItem.title = text(english: "Start typing", chinese: "开始输入")
        cancelItem.title = text(english: "Stop typing", chinese: "停止输入")
        settingsItem.title = text(english: "Open Settings…", chinese: "打开设置…")
        enabledItem.title = text(english: "Enabled", chinese: "启用")
        aboutItem.title = text(english: "About ClipType", chinese: "关于 ClipType")
        quitItem.title = text(english: "Quit ClipType", chinese: "退出 ClipType")
    }

    func update(
        snapshot: RustSnapshot?,
        permission: String,
        startup: String,
        hotkeysRegistered: Bool,
        bridgeAvailable: Bool
    ) {
        enabledItem.state = snapshot?.enabled == true ? .on : .off
        enabledItem.isEnabled = bridgeAvailable

        let mode = snapshot.map { modeName($0.mode) }
            ?? text(english: "Unavailable", chinese: "不可用")
        modeItem.title = "\(text(english: "Mode", chinese: "模式")): \(mode)"

        if permission == "granted" {
            permissionItem.title = text(
                english: "Accessibility: Granted — Manage…",
                chinese: "辅助功能：已授权 — 管理…"
            )
        } else {
            permissionItem.title = text(
                english: "Grant Accessibility Permission…",
                chinese: "授予辅助功能权限…"
            )
        }

        switch startup {
        case "enabled":
            startupItem.state = .on
            startupItem.title = text(english: "Start at Login", chinese: "登录时启动")
        case "requires_approval":
            startupItem.state = .mixed
            startupItem.title = text(
                english: "Start at Login — Needs Approval…",
                chinese: "登录时启动 — 需要批准…"
            )
        case "unsupported":
            startupItem.state = .off
            startupItem.title = text(
                english: "Start at Login — Unsupported",
                chinese: "登录时启动 — 不支持"
            )
        default:
            startupItem.state = .off
            startupItem.title = text(english: "Start at Login", chinese: "登录时启动")
        }
        startupItem.isEnabled = startup != "unsupported" && bridgeAvailable

        let phase = snapshot?.phase ?? CT_BRIDGE_PHASE_IDLE
        let active = phase != CT_BRIDGE_PHASE_IDLE
        let cancelling = phase == CT_BRIDGE_PHASE_CANCELLING
        let ready = bridgeAvailable
            && snapshot?.enabled == true
            && permission == "granted"
            && hotkeysRegistered
            && !active

        triggerItem.isEnabled = ready
        cancelItem.isEnabled = active && !cancelling

        let state = stateName(
            snapshot: snapshot,
            permission: permission,
            hotkeysRegistered: hotkeysRegistered,
            bridgeAvailable: bridgeAvailable
        )
        stateItem.title = "ClipType — \(state)"
        statusItem.button?.toolTip = "ClipType — \(state)"
    }

    private func stateName(
        snapshot: RustSnapshot?,
        permission: String,
        hotkeysRegistered: Bool,
        bridgeAvailable: Bool
    ) -> String {
        guard bridgeAvailable else {
            return text(english: "Runtime Unavailable", chinese: "运行时不可用")
        }
        guard let snapshot else {
            return text(english: "Unavailable", chinese: "不可用")
        }
        switch snapshot.phase {
        case CT_BRIDGE_PHASE_PREPARING:
            return text(english: "Preparing", chinese: "准备中")
        case CT_BRIDGE_PHASE_INJECTING:
            return text(english: "Typing", chinese: "输入中")
        case CT_BRIDGE_PHASE_CANCELLING:
            return text(english: "Stopping", chinese: "停止中")
        default:
            break
        }
        guard snapshot.enabled else {
            return text(english: "Disabled", chinese: "已暂停")
        }
        guard permission == "granted" else {
            return text(english: "Permission Required", chinese: "需要授权")
        }
        guard hotkeysRegistered else {
            return text(english: "Shortcut Unavailable", chinese: "快捷键不可用")
        }
        return text(english: "Ready", chinese: "就绪")
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
        case CT_BRIDGE_MODE_KEYBOARD: return text(english: "Keyboard", chinese: "逐字输入")
        case CT_BRIDGE_MODE_CLIPBOARD: return text(english: "Clipboard", chinese: "快速粘贴")
        case CT_BRIDGE_MODE_CODE: return text(english: "Code", chinese: "代码输入")
        default: return text(english: "Auto", chinese: "自动选择")
        }
    }

    private func text(english: String, chinese: String) -> String {
        language == "zh" ? chinese : english
    }

    private static func makeTemplateStatusImage() -> NSImage {
        let image = NSImage(size: NSSize(width: 18, height: 18), flipped: false) { _ in
            NSColor.black.setStroke()
            let clipboard = NSBezierPath(roundedRect: NSRect(x: 2.5, y: 2.5, width: 9.5, height: 12.5), xRadius: 2, yRadius: 2)
            clipboard.lineWidth = 1.5
            clipboard.stroke()

            let clip = NSBezierPath(roundedRect: NSRect(x: 5, y: 13.2, width: 4.5, height: 2.3), xRadius: 1, yRadius: 1)
            clip.lineWidth = 1.5
            clip.stroke()

            let caret = NSBezierPath()
            caret.move(to: NSPoint(x: 14.8, y: 4))
            caret.line(to: NSPoint(x: 14.8, y: 14))
            caret.move(to: NSPoint(x: 13.3, y: 14))
            caret.line(to: NSPoint(x: 16.3, y: 14))
            caret.move(to: NSPoint(x: 13.3, y: 4))
            caret.line(to: NSPoint(x: 16.3, y: 4))
            caret.lineWidth = 1.5
            caret.stroke()
            return true
        }
        image.isTemplate = true
        return image
    }
}
