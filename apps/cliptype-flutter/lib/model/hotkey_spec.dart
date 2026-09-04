import 'package:flutter/services.dart';

class HotkeySpec {
  const HotkeySpec({required this.modifiers, required this.key});

  final List<String> modifiers;
  final String key;

  String get canonical => [...modifiers, key].join('+');

  String get label {
    final symbols = <String, String>{
      'ctrl': '⌃',
      'alt': '⌥',
      'shift': '⇧',
      'meta': '⌘',
    };
    return '${modifiers.map((modifier) => symbols[modifier]).join()}${_displayKey(key)}';
  }

  static HotkeySpec? fromKeyEvent(KeyEvent event) {
    if (event is! KeyDownEvent || _isModifier(event.logicalKey)) {
      return null;
    }
    final key = _keyName(event.logicalKey);
    if (key == null) {
      return null;
    }
    final keyboard = HardwareKeyboard.instance;
    final modifiers = <String>[
      if (keyboard.isControlPressed) 'ctrl',
      if (keyboard.isAltPressed) 'alt',
      if (keyboard.isShiftPressed) 'shift',
      if (keyboard.isMetaPressed) 'meta',
    ];
    if (!modifiers.any(
      (modifier) =>
          modifier == 'ctrl' || modifier == 'alt' || modifier == 'meta',
    )) {
      return null;
    }
    return HotkeySpec(modifiers: modifiers, key: key);
  }

  static bool isEscape(KeyEvent event) {
    return event.logicalKey == LogicalKeyboardKey.escape;
  }

  static bool isClear(KeyEvent event) {
    return event.logicalKey == LogicalKeyboardKey.delete ||
        event.logicalKey == LogicalKeyboardKey.backspace;
  }

  static bool _isModifier(LogicalKeyboardKey key) {
    return key == LogicalKeyboardKey.controlLeft ||
        key == LogicalKeyboardKey.controlRight ||
        key == LogicalKeyboardKey.altLeft ||
        key == LogicalKeyboardKey.altRight ||
        key == LogicalKeyboardKey.shiftLeft ||
        key == LogicalKeyboardKey.shiftRight ||
        key == LogicalKeyboardKey.metaLeft ||
        key == LogicalKeyboardKey.metaRight;
  }

  static String? _keyName(LogicalKeyboardKey key) {
    final keys = <LogicalKeyboardKey, String>{
      LogicalKeyboardKey.keyA: 'a',
      LogicalKeyboardKey.keyB: 'b',
      LogicalKeyboardKey.keyC: 'c',
      LogicalKeyboardKey.keyD: 'd',
      LogicalKeyboardKey.keyE: 'e',
      LogicalKeyboardKey.keyF: 'f',
      LogicalKeyboardKey.keyG: 'g',
      LogicalKeyboardKey.keyH: 'h',
      LogicalKeyboardKey.keyI: 'i',
      LogicalKeyboardKey.keyJ: 'j',
      LogicalKeyboardKey.keyK: 'k',
      LogicalKeyboardKey.keyL: 'l',
      LogicalKeyboardKey.keyM: 'm',
      LogicalKeyboardKey.keyN: 'n',
      LogicalKeyboardKey.keyO: 'o',
      LogicalKeyboardKey.keyP: 'p',
      LogicalKeyboardKey.keyQ: 'q',
      LogicalKeyboardKey.keyR: 'r',
      LogicalKeyboardKey.keyS: 's',
      LogicalKeyboardKey.keyT: 't',
      LogicalKeyboardKey.keyU: 'u',
      LogicalKeyboardKey.keyV: 'v',
      LogicalKeyboardKey.keyW: 'w',
      LogicalKeyboardKey.keyX: 'x',
      LogicalKeyboardKey.keyY: 'y',
      LogicalKeyboardKey.keyZ: 'z',
      LogicalKeyboardKey.digit0: '0',
      LogicalKeyboardKey.digit1: '1',
      LogicalKeyboardKey.digit2: '2',
      LogicalKeyboardKey.digit3: '3',
      LogicalKeyboardKey.digit4: '4',
      LogicalKeyboardKey.digit5: '5',
      LogicalKeyboardKey.digit6: '6',
      LogicalKeyboardKey.digit7: '7',
      LogicalKeyboardKey.digit8: '8',
      LogicalKeyboardKey.digit9: '9',
      LogicalKeyboardKey.f1: 'f1',
      LogicalKeyboardKey.f2: 'f2',
      LogicalKeyboardKey.f3: 'f3',
      LogicalKeyboardKey.f4: 'f4',
      LogicalKeyboardKey.f5: 'f5',
      LogicalKeyboardKey.f6: 'f6',
      LogicalKeyboardKey.f7: 'f7',
      LogicalKeyboardKey.f8: 'f8',
      LogicalKeyboardKey.f9: 'f9',
      LogicalKeyboardKey.f10: 'f10',
      LogicalKeyboardKey.f11: 'f11',
      LogicalKeyboardKey.f12: 'f12',
      LogicalKeyboardKey.f13: 'f13',
      LogicalKeyboardKey.f14: 'f14',
      LogicalKeyboardKey.f15: 'f15',
      LogicalKeyboardKey.f16: 'f16',
      LogicalKeyboardKey.f17: 'f17',
      LogicalKeyboardKey.f18: 'f18',
      LogicalKeyboardKey.f19: 'f19',
      LogicalKeyboardKey.f20: 'f20',
      LogicalKeyboardKey.f21: 'f21',
      LogicalKeyboardKey.f22: 'f22',
      LogicalKeyboardKey.f23: 'f23',
      LogicalKeyboardKey.f24: 'f24',
      LogicalKeyboardKey.space: 'space',
      LogicalKeyboardKey.tab: 'tab',
      LogicalKeyboardKey.enter: 'enter',
      LogicalKeyboardKey.insert: 'insert',
      LogicalKeyboardKey.delete: 'delete',
      LogicalKeyboardKey.home: 'home',
      LogicalKeyboardKey.end: 'end',
      LogicalKeyboardKey.pageUp: 'pageup',
      LogicalKeyboardKey.pageDown: 'pagedown',
      LogicalKeyboardKey.arrowLeft: 'left',
      LogicalKeyboardKey.arrowRight: 'right',
      LogicalKeyboardKey.arrowUp: 'up',
      LogicalKeyboardKey.arrowDown: 'down',
      LogicalKeyboardKey.minus: 'minus',
      LogicalKeyboardKey.equal: 'equal',
      LogicalKeyboardKey.bracketLeft: 'bracket-left',
      LogicalKeyboardKey.bracketRight: 'bracket-right',
      LogicalKeyboardKey.backslash: 'backslash',
      LogicalKeyboardKey.semicolon: 'semicolon',
      LogicalKeyboardKey.quote: 'quote',
      LogicalKeyboardKey.comma: 'comma',
      LogicalKeyboardKey.period: 'period',
      LogicalKeyboardKey.slash: 'slash',
      LogicalKeyboardKey.backquote: 'backquote',
    };
    return keys[key];
  }

  static String _displayKey(String key) {
    const display = <String, String>{
      'space': 'Space',
      'tab': 'Tab',
      'enter': 'Return',
      'pageup': 'Page Up',
      'pagedown': 'Page Down',
      'bracket-left': '[',
      'bracket-right': ']',
      'backslash': '\\',
      'semicolon': ';',
      'quote': "'",
      'comma': ',',
      'period': '.',
      'slash': '/',
      'backquote': '`',
      'minus': '-',
      'equal': '=',
    };
    return display[key] ?? key.toUpperCase();
  }
}
