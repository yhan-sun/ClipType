import Cocoa
import FlutterMacOS

@main
class AppDelegate: FlutterAppDelegate {
  private var nativePlugin: ClipTypeNativePlugin?
  private var terminationRequested = false

  override func applicationWillFinishLaunching(_ notification: Notification) {
    super.applicationWillFinishLaunching(notification)
    // ClipType remains available from the menu bar while its settings window
    // is hidden. It is still one ordinary, unprivileged application process.
    NSApp.setActivationPolicy(.accessory)
  }

  func attachFlutter(_ viewController: FlutterViewController, window: MainFlutterWindow) {
    mainFlutterWindow = window
    if nativePlugin == nil {
      nativePlugin = ClipTypeNativePlugin(appDelegate: self)
    }
    nativePlugin?.attach(to: viewController)

    if CommandLine.arguments.contains("--settings") {
      DispatchQueue.main.async { [weak self] in
        self?.showSettings()
      }
    }
  }

  func showSettings() {
    guard let window = mainFlutterWindow else { return }
    window.makeKeyAndOrderFront(nil)
    NSApp.activate(ignoringOtherApps: true)
  }

  func terminateClipType() {
    guard !terminationRequested else { return }
    terminationRequested = true
    nativePlugin?.shutdown()
    NSApp.terminate(nil)
  }

  override func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
    return false
  }

  override func applicationSupportsSecureRestorableState(_ app: NSApplication) -> Bool {
    return true
  }

  override func applicationWillTerminate(_ notification: Notification) {
    nativePlugin?.shutdown()
    super.applicationWillTerminate(notification)
  }
}
