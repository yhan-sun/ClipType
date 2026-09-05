import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../model/hotkey_spec.dart';

class HotkeyRecorder extends StatefulWidget {
  const HotkeyRecorder({
    required this.title,
    required this.value,
    required this.onChanged,
    required this.onCleared,
    this.enabled = true,
    super.key,
  });

  final String title;
  final String value;
  final ValueChanged<String> onChanged;
  final VoidCallback onCleared;
  final bool enabled;

  @override
  State<HotkeyRecorder> createState() => _HotkeyRecorderState();
}

class _HotkeyRecorderState extends State<HotkeyRecorder> {
  late final FocusNode _focusNode;
  bool _recording = false;

  @override
  void initState() {
    super.initState();
    _focusNode = FocusNode(debugLabel: '${widget.title} shortcut recorder');
  }

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  KeyEventResult _handleKey(FocusNode node, KeyEvent event) {
    if (!_recording) return KeyEventResult.ignored;
    if (HotkeySpec.isEscape(event)) {
      setState(() => _recording = false);
      node.unfocus();
      return KeyEventResult.handled;
    }
    if (HotkeySpec.isClear(event)) {
      widget.onCleared();
      setState(() => _recording = false);
      node.unfocus();
      return KeyEventResult.handled;
    }
    final spec = HotkeySpec.fromKeyEvent(event);
    if (spec == null) {
      // Modifier-only and unknown key presses never complete a recording.
      return KeyEventResult.handled;
    }
    widget.onChanged(spec.canonical);
    setState(() => _recording = false);
    node.unfocus();
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = context.l10n;
    final value = widget.value.isEmpty ? l10n.notSet : _label(widget.value);
    return Focus(
      focusNode: _focusNode,
      onKeyEvent: _handleKey,
      child: Semantics(
        button: true,
        label: l10n.text(
          '${widget.title} shortcut recorder',
          '${widget.title} 快捷键录制器',
        ),
        liveRegion: _recording,
        value: value,
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            InkWell(
              borderRadius: BorderRadius.circular(14),
              onTap: widget.enabled
                  ? () {
                      setState(() => _recording = true);
                      _focusNode.requestFocus();
                    }
                  : null,
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 140),
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  color: _recording
                      ? theme.colorScheme.primaryContainer
                      : theme.colorScheme.surfaceContainerHighest,
                  borderRadius: BorderRadius.circular(14),
                  border: Border.all(
                    color: _recording
                        ? theme.colorScheme.primary
                        : theme.colorScheme.outlineVariant,
                    width: _recording ? 2 : 1,
                  ),
                ),
                child: Row(
                  children: [
                    Icon(
                      _recording ? Icons.radio_button_checked : Icons.keyboard,
                      color: _recording
                          ? theme.colorScheme.primary
                          : theme.colorScheme.onSurfaceVariant,
                    ),
                    const SizedBox(width: 12),
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(widget.title, style: theme.textTheme.labelLarge),
                          const SizedBox(height: 5),
                          Text(
                            _recording ? l10n.recordPrompt : value,
                            style: theme.textTheme.titleMedium?.copyWith(
                              fontWeight: FontWeight.w700,
                            ),
                          ),
                        ],
                      ),
                    ),
                    Text(
                      _recording ? l10n.escapeToCancel : l10n.record,
                      style: theme.textTheme.labelMedium?.copyWith(
                        color: theme.colorScheme.primary,
                      ),
                    ),
                  ],
                ),
              ),
            ),
            Align(
              alignment: Alignment.centerRight,
              child: TextButton(
                onPressed: widget.enabled
                    ? () {
                        widget.onCleared();
                        setState(() => _recording = false);
                        _focusNode.unfocus();
                      }
                    : null,
                child: Text(l10n.clear),
              ),
            ),
          ],
        ),
      ),
    );
  }

  String _label(String value) {
    final tokens = value.split('+');
    if (tokens.length < 2) return value;
    final modifiers = tokens.take(tokens.length - 1).toList();
    final key = tokens.last;
    return HotkeySpec(modifiers: modifiers, key: key).label;
  }
}
