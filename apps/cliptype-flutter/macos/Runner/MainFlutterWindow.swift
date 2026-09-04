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
    // application or unregister its global shortcuts.
    sender.orderOut(nil)
    return false
  }
}
