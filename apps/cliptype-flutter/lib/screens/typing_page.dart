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
    if (oldWidget.controller.settings != widget.controller.settings) {
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
    return ListView(
      padding: const EdgeInsets.fromLTRB(32, 30, 32, 32),
      children: [
        PageHeader(title: l10n.typing, description: l10n.typingDescription),
        const SizedBox(height: 26),
        SettingCard(
          title: l10n.deliveryMode,
          child: DropdownButtonFormField<InjectionMode>(
            key: ValueKey(_draft.mode),
            initialValue: _draft.mode,
            decoration: const InputDecoration(
              labelText: 'Mode',
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
                setState(() => _draft = _draft.copyWith(mode: value));
              }
            },
          ),
        ),
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
                onChanged: (value) => setState(
                  () => _draft = _draft.copyWith(
                    charactersPerSecond: int.tryParse(value) ?? 0,
                  ),
                ),
              ),
              const SizedBox(height: 16),
              _PercentSlider(
                label: l10n.timingJitter,
                value: _draft.jitterPercent,
                max: 95,
                onChanged: (value) => setState(
                  () => _draft = _draft.copyWith(jitterPercent: value.round()),
                ),
              ),
              const SizedBox(height: 12),
              _PercentSlider(
                label: l10n.correctedTypoProbability,
                value: _draft.typoProbabilityPercent,
                max: 25,
                onChanged: (value) => setState(
                  () => _draft = _draft.copyWith(
                    typoProbabilityPercent: value.round(),
                  ),
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
            onChanged: (value) => setState(
              () => _draft = _draft.copyWith(
                autoClipboardThreshold: int.tryParse(value) ?? 0,
              ),
            ),
          ),
        ),
        const SizedBox(height: 16),
        SettingCard(
          child: SaveBar(
            saving: widget.controller.saving,
            error: validation == null
                ? widget.controller.error
                : l10n.validationMessage(validation),
            onReset: () => setState(() {
              _draft = AppSettings.defaults();
              _cps.text = '${_draft.charactersPerSecond}';
              _threshold.text = '${_draft.autoClipboardThreshold}';
            }),
            onApply: () async {
              final applied = await widget.controller.save(_draft);
              if (applied && mounted) setState(() {});
            },
          ),
        ),
      ],
    );
  }
}

class _NumberField extends StatelessWidget {
  const _NumberField({
    required this.controller,
    required this.label,
    required this.helper,
    required this.onChanged,
  });

  final TextEditingController controller;
  final String label;
  final String helper;
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
    required this.onChanged,
  });

  final String label;
  final int value;
  final int max;
  final ValueChanged<double> onChanged;

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Row(
          children: [
            Expanded(child: Text(label)),
            Text('$value%'),
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
