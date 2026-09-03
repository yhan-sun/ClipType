import AppKit
import Foundation

private struct Fixture {
    let identifier: String
    let text: String
}

private let fixtures: [Fixture] = [
    Fixture(identifier: "ascii", text: "ClipType-P3-S01-ASCII-0123456789"),
    Fixture(identifier: "cjk", text: "中文输入_日本語入力_한국어입력"),
    Fixture(identifier: "emoji", text: "emoji_😀_🧑🏽‍💻_🚀"),
    Fixture(identifier: "combining", text: "e\u{0301}_a\u{0300}_o\u{0302}_u\u{0308}_n\u{0303}_c\u{0327}"),
    Fixture(identifier: "multiline", text: "line-1\nline-2\nline-3"),
    Fixture(identifier: "tab", text: "column-a\tcolumn-b\tcolumn-c"),
    Fixture(identifier: "long", text: String(repeating: "ClipType-P3-S01-long-fixture-", count: 96)),
]

@main
private final class P3S01Target: NSObject, NSApplicationDelegate {
    private var window: NSWindow!
    private var fieldA: NSTextView!
    private var fieldB: NSTextView!
    private var fixturePicker: NSPopUpButton!
    private var statusLabel: NSTextField!
    private let resultPath = ProcessInfo.processInfo.environment["P3_S01_RESULTS_JSONL"]

    static func main() {
        let app = NSApplication.shared
        let delegate = P3S01Target()
        app.delegate = delegate
        app.setActivationPolicy(.regular)
        app.run()
    }

    func applicationDidFinishLaunching(_ notification: Notification) {
        createWindow()
        window.makeKeyAndOrderFront(nil)
        NSApp.activate(ignoringOtherApps: true)
        focusA()
    }

    func applicationShouldTerminateAfterLastWindowClosed(_ sender: NSApplication) -> Bool {
        true
    }

    private func createWindow() {
        let frame = NSRect(x: 0, y: 0, width: 940, height: 650)
        window = NSWindow(
            contentRect: frame,
            styleMask: [.titled, .closable, .miniaturizable, .resizable],
            backing: .buffered,
            defer: false
        )
        window.title = "ClipType P3-S01 Controlled Native Target"
        window.center()
        window.minSize = NSSize(width: 760, height: 520)

        guard let content = window.contentView else {
            fatalError("window content view unavailable")
        }

        let top = NSStackView()
        top.orientation = .horizontal
        top.spacing = 8
        top.alignment = .centerY
        top.translatesAutoresizingMaskIntoConstraints = false

        fixturePicker = NSPopUpButton()
        fixturePicker.addItems(withTitles: fixtures.map(\.identifier))
        top.addArrangedSubview(label("Fixture:"))
        top.addArrangedSubview(fixturePicker)
        top.addArrangedSubview(button("Copy fixture", #selector(copyFixture)))
        top.addArrangedSubview(button("Focus A", #selector(focusA)))
        top.addArrangedSubview(button("Focus B", #selector(focusB)))
        top.addArrangedSubview(button("Clear", #selector(clearFields)))
        top.addArrangedSubview(button("Verify A", #selector(verifyA)))
        top.addArrangedSubview(button("Verify B", #selector(verifyB)))

        let columns = NSStackView()
        columns.orientation = .horizontal
        columns.distribution = .fillEqually
        columns.spacing = 14
        columns.translatesAutoresizingMaskIntoConstraints = false

        fieldA = textView()
        fieldB = textView()
        columns.addArrangedSubview(column(title: "Native NSTextView A", textView: fieldA))
        columns.addArrangedSubview(column(title: "Native NSTextView B", textView: fieldB))

        statusLabel = NSTextField(labelWithString: "Ready. Evidence output contains counts and PASS/FAIL only.")
        statusLabel.lineBreakMode = .byWordWrapping
        statusLabel.maximumNumberOfLines = 3
        statusLabel.translatesAutoresizingMaskIntoConstraints = false

        content.addSubview(top)
        content.addSubview(columns)
        content.addSubview(statusLabel)

        NSLayoutConstraint.activate([
            top.leadingAnchor.constraint(equalTo: content.leadingAnchor, constant: 16),
            top.trailingAnchor.constraint(lessThanOrEqualTo: content.trailingAnchor, constant: -16),
            top.topAnchor.constraint(equalTo: content.topAnchor, constant: 16),

            columns.leadingAnchor.constraint(equalTo: content.leadingAnchor, constant: 16),
            columns.trailingAnchor.constraint(equalTo: content.trailingAnchor, constant: -16),
            columns.topAnchor.constraint(equalTo: top.bottomAnchor, constant: 14),
            columns.bottomAnchor.constraint(equalTo: statusLabel.topAnchor, constant: -12),

            statusLabel.leadingAnchor.constraint(equalTo: content.leadingAnchor, constant: 16),
            statusLabel.trailingAnchor.constraint(equalTo: content.trailingAnchor, constant: -16),
            statusLabel.bottomAnchor.constraint(equalTo: content.bottomAnchor, constant: -16),
            statusLabel.heightAnchor.constraint(greaterThanOrEqualToConstant: 36),
        ])
    }

    private func label(_ value: String) -> NSTextField {
        NSTextField(labelWithString: value)
    }

    private func button(_ title: String, _ action: Selector) -> NSButton {
        let result = NSButton(title: title, target: self, action: action)
        result.bezelStyle = .rounded
        return result
    }

    private func textView() -> NSTextView {
        let result = NSTextView(frame: .zero)
        result.isRichText = false
        result.isAutomaticQuoteSubstitutionEnabled = false
        result.isAutomaticDashSubstitutionEnabled = false
        result.isAutomaticTextReplacementEnabled = false
        result.isAutomaticSpellingCorrectionEnabled = false
        result.isContinuousSpellCheckingEnabled = false
        result.allowsUndo = true
        result.font = NSFont.monospacedSystemFont(ofSize: 15, weight: .regular)
        return result
    }

    private func column(title: String, textView: NSTextView) -> NSView {
        let stack = NSStackView()
        stack.orientation = .vertical
        stack.spacing = 6
        let heading = NSTextField(labelWithString: title)
        heading.font = NSFont.boldSystemFont(ofSize: 13)
        let scroll = NSScrollView()
        scroll.hasVerticalScroller = true
        scroll.hasHorizontalScroller = true
        scroll.borderType = .bezelBorder
        scroll.documentView = textView
        textView.minSize = NSSize(width: 0, height: 0)
        textView.maxSize = NSSize(
            width: CGFloat.greatestFiniteMagnitude,
            height: CGFloat.greatestFiniteMagnitude
        )
        textView.isVerticallyResizable = true
        textView.isHorizontallyResizable = true
        textView.autoresizingMask = [.width]
        textView.textContainer?.containerSize = NSSize(
            width: CGFloat.greatestFiniteMagnitude,
            height: CGFloat.greatestFiniteMagnitude
        )
        textView.textContainer?.widthTracksTextView = false
        stack.addArrangedSubview(heading)
        stack.addArrangedSubview(scroll)
        return stack
    }

    private var selectedFixture: Fixture {
        fixtures[max(0, fixturePicker.indexOfSelectedItem)]
    }

    @objc private func copyFixture() {
        let fixture = selectedFixture
        let pasteboard = NSPasteboard.general
        pasteboard.clearContents()
        let copied = pasteboard.setString(fixture.text, forType: .string)
        statusLabel.stringValue = copied
            ? "Fixture \(fixture.identifier) copied; focus returned to A."
            : "Clipboard write failed for fixture \(fixture.identifier)."
        focusA()
    }

    @objc private func focusA() {
        window.makeKeyAndOrderFront(nil)
        window.makeFirstResponder(fieldA)
    }

    @objc private func focusB() {
        window.makeKeyAndOrderFront(nil)
        window.makeFirstResponder(fieldB)
    }

    @objc private func clearFields() {
        fieldA.string = ""
        fieldB.string = ""
        statusLabel.stringValue = "Both controlled fields cleared."
        focusA()
    }

    @objc private func verifyA() {
        verify(field: "A", value: fieldA.string)
    }

    @objc private func verifyB() {
        verify(field: "B", value: fieldB.string)
    }

    private func verify(field: String, value: String) {
        let fixture = selectedFixture
        let passed = value == fixture.text
        let record: [String: Any] = [
            "schema": 1,
            "case_id": "TARGET_\(fixture.identifier.uppercased())_\(field)",
            "fixture_class": fixture.identifier,
            "field": field,
            "result": passed ? "PASS" : "FAIL",
            "expected_utf8_bytes": fixture.text.utf8.count,
            "expected_scalars": fixture.text.unicodeScalars.count,
            "observed_utf8_bytes": value.utf8.count,
            "observed_scalars": value.unicodeScalars.count,
        ]
        appendResult(record)
        statusLabel.stringValue = [
            passed ? "PASS" : "FAIL",
            "fixture=\(fixture.identifier)",
            "field=\(field)",
            "expected_bytes=\(fixture.text.utf8.count)",
            "observed_bytes=\(value.utf8.count)",
        ].joined(separator: " ")
    }

    private func appendResult(_ record: [String: Any]) {
        guard let resultPath else {
            return
        }
        guard JSONSerialization.isValidJSONObject(record),
              let data = try? JSONSerialization.data(withJSONObject: record, options: [.sortedKeys])
        else {
            statusLabel.stringValue = "Result serialization failed."
            return
        }
        let url = URL(fileURLWithPath: resultPath)
        _ = FileManager.default.createFile(atPath: url.path, contents: nil)
        guard let handle = try? FileHandle(forWritingTo: url) else {
            statusLabel.stringValue = "Result file could not be opened."
            return
        }
        defer { try? handle.close() }
        do {
            try handle.seekToEnd()
            try handle.write(contentsOf: data)
            try handle.write(contentsOf: Data([0x0A]))
        } catch {
            statusLabel.stringValue = "Result file could not be updated."
        }
    }
}
