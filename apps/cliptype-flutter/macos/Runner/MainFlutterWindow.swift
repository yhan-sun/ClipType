import Cocoa
import FlutterMacOS

class MainFlutterWindow: NSWindow, NSWindowDelegate {
  private(set) var flutterViewController: FlutterViewController?

  override func awakeFromNib() {
    let flutterViewController = FlutterViewController()
    self.flutterViewController = flutterViewController
    let windowFrame = self.frame
    self.minSize = NSSize(width: 720, height: 520)
    self.contentViewController = flutterViewController
    self.setFrame(windowFrame, display: true)

    RegisterGeneratedPlugins(registry: flutterViewController)

    super.awakeFromNib()
    delegate = self
    (NSApp.delegate as? AppDelegate)?.attachFlutter(flutterViewController, window: self)
  }

  func windowShouldClose(_ sender: NSWindow) -> Bool {
    // Closing Settings hides the window but must not terminate the menu-bar
    // application or unregister its global shortcuts. Explain this once so a
    // first-time user does not mistake the hidden settings window for a quit.
    let defaults = UserDefaults.standard
    let key = "ClipType.didExplainMenuBarPersistence"
    if !defaults.bool(forKey: key) {
      let language = defaults.string(forKey: "ClipType.interfaceLanguage") == "zh" ? "zh" : "en"
      let alert = NSAlert()
      alert.messageText = language == "zh" ? "ClipType 仍在菜单栏运行" : "ClipType stays in the menu bar"
      alert.informativeText = language == "zh"
        ? "关闭设置窗口不会退出 ClipType。之后可以点击菜单栏图标重新打开设置。"
        : "Closing Settings does not quit ClipType. Use the menu-bar icon to reopen Settings later."
      alert.addButton(withTitle: language == "zh" ? "知道了" : "OK")
      alert.runModal()
      defaults.set(true, forKey: key)
    }
    sender.orderOut(nil)
    return false
  }
}
