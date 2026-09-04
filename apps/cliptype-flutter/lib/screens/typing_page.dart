import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../l10n/app_localizations.dart';
import '../model/app_settings.dart';
import '../state/settings_controller.dart';
import '../widgets/setting_card.dart';

class TypingPage extends StatefulWidget {
  const TypingPage({required this.controller, super.key});

  final SettingsController controller;

  @override
  State<TypingPage> createState() => _TypingPageState();
}

class _TypingPageState extends State<TypingPage> {
  late AppSettings _draft;
  late final TextEditingController _cps;
  late final TextEditingController _threshold;
  bool _hasLocalDraft = false;

  @override
  void initState() {
    super.initState();
    _draft = widget.controller.settings;
    _cps = TextEditingController(text: '${_draft.charactersPerSecond}');
    _threshold = TextEditingController(
      text: '${_draft.autoClipboardThreshold}',
    );
  }

  @override
  void didUpdateWidget(covariant TypingPage oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.controller != widget.controller) {
      _draft = widget.controller.settings;
      _cps.text = '${_draft.charactersPerSecond}';
      _threshold.text = '${_draft.autoClipboardThreshold}';
      _hasLocalDraft = false;
      return;
    }
    if (_hasLocalDraft && _draft == widget.controller.settings) {
      _hasLocalDraft = false;
    }
    if (!_hasLocalDraft && _draft != widget.controller.settings) {
      _draft = widget.controller.settings;
      _cps.text = '${_draft.charactersPerSecond}';
      _threshold.text = '${_draft.autoClipboardThreshold}';
    }
  }

  @override
  void dispose() {
    _cps.dispose();
    _threshold.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final validation = _draft.validationCode();
    return PageContent(
      children: [
        PageHeader(
          title: l10n.typing,
          description: l10n.typingDescription,
          trailing: AutoSaveIndicator(controller: widget.controller),
        ),
        const SizedBox(height: 26),
        SettingCard(
          title: l10n.deliveryMode,
          child: DropdownButtonFormField<InjectionMode>(
            key: ValueKey(_draft.mode),
            initialValue: _draft.mode,
            decoration: InputDecoration(
              labelText: l10n.mode,
              border: OutlineInputBorder(),
            ),
            items: InjectionMode.values
                .map(
                  (mode) => DropdownMenuItem(
                    value: mode,
                    child: Text(l10n.modeLabel(mode)),
                  ),
                )
                .toList(),
            onChanged: (value) {
              if (value != null) {
                _updateDraft(_draft.copyWith(mode: value), immediate: true);
              }
            },
          ),
        ),
        if (_draft.mode == InjectionMode.code) ...[
          const SizedBox(height: 12),
          SettingCard(child: Text(l10n.codeModeSubtitle)),
        ],
        const SizedBox(height: 16),
        SettingCard(
          title: l10n.humanPacedControls,
          subtitle: l10n.keyboardControlsSubtitle,
          child: Column(
            children: [
              _NumberField(
                controller: _cps,
                label: l10n.charactersPerSecond,
                helper: l10n.charactersPerSecondHelper,
                errorText: validation == 'characters_per_second'
                    ? l10n.validationMessage(validation)
                    : null,
                onChanged: (value) => _updateDraft(
                  _draft.copyWith(
                    charactersPerSecond: int.tryParse(value) ?? 0,
                  ),
                ),
              ),
              const SizedBox(height: 16),
              _PercentSlider(
                label: l10n.timingJitter,
                value: _draft.jitterPercent,
                max: 95,
                onChanged: (value) =>
                    _updateDraft(_draft.copyWith(jitterPercent: value.round())),
              ),
              const SizedBox(height: 12),
              _PercentSlider(
                label: l10n.correctedTypoProbability,
                value: _draft.typoProbabilityPercent,
                max: 25,
                onChanged: _draft.mode == InjectionMode.code
                    ? null
                    : (value) => _updateDraft(
                        _draft.copyWith(typoProbabilityPercent: value.round()),
                      ),
              ),
              if (_draft.typoProbabilityPercent > 0) ...[
                const SizedBox(height: 12),
                Container(
                  width: double.infinity,
                  padding: const EdgeInsets.all(12),
                  decoration: BoxDecoration(
                    color: Theme.of(context).colorScheme.errorContainer,
                    borderRadius: BorderRadius.circular(10),
                  ),
                  child: Text(
                    l10n.typoWarning,
                    style: TextStyle(
                      color: Theme.of(context).colorScheme.onErrorContainer,
                    ),
                  ),
                ),
              ],
            ],
          ),
        ),
        const SizedBox(height: 16),
        SettingCard(
          title: l10n.autoPolicy,
          subtitle: l10n.autoPolicySubtitle,
          child: _NumberField(
            controller: _threshold,
            label: l10n.autoClipboardThreshold,
            helper: l10n.semanticElementsMinimum,
            errorText: validation == 'auto_clipboard_threshold'
                ? l10n.validationMessage(validation)
                : null,
            onChanged: (value) => _updateDraft(
              _draft.copyWith(autoClipboardThreshold: int.tryParse(value) ?? 0),
            ),
          ),
        ),
        const SizedBox(height: 16),
        AutoSaveFooter(
          controller: widget.controller,
          onReset: _restoreDefaults,
        ),
      ],
    );
  }

  void _updateDraft(AppSettings next, {bool immediate = false}) {
    setState(() {
      _draft = next;
      _hasLocalDraft = true;
    });
    widget.controller.updateSettings(
      next,
      debounce: immediate ? Duration.zero : SettingsController.autoSaveDebounce,
    );
  }

  void _restoreDefaults() {
    final defaults = AppSettings.defaults();
    final next = _draft.copyWith(
      mode: defaults.mode,
      charactersPerSecond: defaults.charactersPerSecond,
      jitterPercent: defaults.jitterPercent,
      typoProbabilityPercent: defaults.typoProbabilityPercent,
      autoClipboardThreshold: defaults.autoClipboardThreshold,
    );
    _updateDraft(next, immediate: true);
    _cps.text = '${next.charactersPerSecond}';
    _threshold.text = '${next.autoClipboardThreshold}';
  }
}

class _NumberField extends StatelessWidget {
  const _NumberField({
    required this.controller,
    required this.label,
    required this.helper,
    required this.onChanged,
    this.errorText,
  });

  final TextEditingController controller;
  final String label;
  final String helper;
  final String? errorText;
  final ValueChanged<String> onChanged;

  @override
  Widget build(BuildContext context) {
    return TextField(
      controller: controller,
      keyboardType: TextInputType.number,
      inputFormatters: [FilteringTextInputFormatter.digitsOnly],
      onChanged: onChanged,
      decoration: InputDecoration(
        labelText: label,
        helperText: helper,
        errorText: errorText,
        border: const OutlineInputBorder(),
      ),
    );
  }
}

class _PercentSlider extends StatelessWidget {
  const _PercentSlider({
    required this.label,
    required this.value,
    required this.max,
    this.onChanged,
  });

  final String label;
  final int value;
  final int max;
  final ValueChanged<double>? onChanged;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Row(
          children: [
            Expanded(
              child: Text(
                label,
                style: onChanged == null
                    ? TextStyle(color: Theme.of(context).disabledColor)
                    : null,
              ),
            ),
            Text(
              '$value%',
              style: onChanged == null
                  ? TextStyle(color: Theme.of(context).disabledColor)
                  : null,
            ),
          ],
        ),
        Slider(
          value: value.toDouble().clamp(0, max).toDouble(),
          min: 0,
          max: max.toDouble(),
          divisions: max,
          label: '$value%',
          onChanged: onChanged,
        ),
      ],
    );
  }
}
