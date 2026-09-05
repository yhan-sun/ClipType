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
  String? _cpsError;
  String? _thresholdError;

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
      _syncFromController();
      return;
    }
    if (_hasLocalDraft && _draft == widget.controller.settings) {
      _hasLocalDraft = false;
    }
    if (!_hasLocalDraft && _draft != widget.controller.settings) {
      _syncFromController();
    }
  }

  void _syncFromController() {
    _draft = widget.controller.settings;
    _cps.text = '${_draft.charactersPerSecond}';
    _threshold.text = '${_draft.autoClipboardThreshold}';
    _cpsError = null;
    _thresholdError = null;
    _hasLocalDraft = false;
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
    final keyboardPacing = switch (_draft.mode) {
      InjectionMode.keyboard ||
      InjectionMode.code ||
      InjectionMode.auto => true,
      InjectionMode.clipboard => false,
    };
    return PageContent(
      children: [
        PageHeader(
          title: l10n.text('Input', '输入'),
          description: l10n.text(
            'Choose how ClipType delivers text. Only settings that affect the selected mode are shown.',
            '选择 ClipType 的输入方式。页面只显示对当前模式真正生效的设置。',
          ),
        ),
        const SizedBox(height: 24),
        SettingCard(
          title: l10n.deliveryMode,
          subtitle: l10n.text(
            'Auto is recommended for general use. Code is designed for editors with ordinary auto-pair and auto-indent enabled.',
            '日常使用推荐“自动选择”；“代码输入”适用于已开启常规自动配对和自动缩进的编辑器。',
          ),
          child: LayoutBuilder(
            builder: (context, constraints) {
              final width = constraints.maxWidth < 620
                  ? constraints.maxWidth
                  : (constraints.maxWidth - 12) / 2;
              return Wrap(
                spacing: 12,
                runSpacing: 12,
                children: InjectionMode.values
                    .map(
                      (mode) => SizedBox(
                        width: width,
                        child: _ModeCard(
                          mode: mode,
                          selected: _draft.mode == mode,
                          onTap: () => _updateDraft(
                            _draft.copyWith(mode: mode),
                            immediate: true,
                          ),
                        ),
                      ),
                    )
                    .toList(),
              );
            },
          ),
        ),
        if (_draft.mode == InjectionMode.code) ...[
          const SizedBox(height: 12),
          _InfoPanel(icon: Icons.code_rounded, text: l10n.codeModeSubtitle),
        ],
        if (_draft.mode == InjectionMode.clipboard) ...[
          const SizedBox(height: 12),
          _InfoPanel(
            icon: Icons.bolt_rounded,
            text: l10n.text(
              'Clipboard mode sends one guarded ordinary paste action. Typing speed, jitter, and corrected-typo settings do not apply.',
              '剪贴板模式只发送一次受保护的普通粘贴动作；输入速度、时间抖动和纠错错字设置均不生效。',
            ),
          ),
        ],
        if (_draft.mode == InjectionMode.auto) ...[
          const SizedBox(height: 16),
          SettingCard(
            title: l10n.text('Automatic selection', '自动选择策略'),
            subtitle: l10n.text(
              'Auto prefers guarded paste for non-ASCII text and uses this size threshold for otherwise keyboard-friendly text.',
              '自动模式会优先为非 ASCII 文本选择受保护粘贴；对于适合逐字输入的文本，则使用下面的长度阈值进行选择。',
            ),
            child: _NumberField(
              controller: _threshold,
              label: l10n.autoClipboardThreshold,
              helper: l10n.text(
                'Minimum 1. Empty or incomplete edits are not saved.',
                '最小值为 1；输入为空或尚未完成时不会保存。',
              ),
              errorText: _thresholdError,
              onChanged: _onThresholdChanged,
            ),
          ),
        ],
        if (keyboardPacing) ...[
          const SizedBox(height: 16),
          SettingCard(
            title: _draft.mode == InjectionMode.auto
                ? l10n.text('Keyboard fallback pacing', '逐字输入回退节奏')
                : l10n.humanPacedControls,
            subtitle: _draft.mode == InjectionMode.auto
                ? l10n.text(
                    'These values apply only when Auto selects the keyboard backend.',
                    '这些设置只在自动模式最终选择逐字输入时生效。',
                  )
                : l10n.keyboardControlsSubtitle,
            child: Column(
              children: [
                _NumberField(
                  controller: _cps,
                  label: l10n.charactersPerSecond,
                  helper: l10n.text(
                    '1–250 actions per second. Empty or incomplete edits are not saved.',
                    '每秒 1–250 个动作；输入为空或尚未完成时不会保存。',
                  ),
                  errorText: _cpsError,
                  onChanged: _onCpsChanged,
                ),
                const SizedBox(height: 8),
                Align(
                  alignment: Alignment.centerLeft,
                  child: Text(
                    _speedSummary(l10n),
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Theme.of(context).colorScheme.onSurfaceVariant,
                    ),
                  ),
                ),
                const SizedBox(height: 18),
                _PercentSlider(
                  label: l10n.timingJitter,
                  value: _draft.jitterPercent,
                  max: 95,
                  onChanged: (value) => _updateDraft(
                    _draft.copyWith(jitterPercent: value.round()),
                  ),
                ),
                const SizedBox(height: 12),
                _PercentSlider(
                  label: l10n.correctedTypoProbability,
                  value: _draft.typoProbabilityPercent,
                  max: 25,
                  onChanged: (value) => _updateDraft(
                    _draft.copyWith(typoProbabilityPercent: value.round()),
                  ),
                ),
                if (_draft.typoProbabilityPercent > 0) ...[
                  const SizedBox(height: 12),
                  Container(
                    width: double.infinity,
                    padding: const EdgeInsets.all(12),
                    decoration: BoxDecoration(
                      color: Theme.of(context).colorScheme.secondaryContainer,
                      borderRadius: BorderRadius.circular(10),
                    ),
                    child: Text(
                      _draft.mode == InjectionMode.code
                          ? l10n.text(
                              'Code mode limits temporary wrong keys to a safe ASCII subset. Brackets, quotes, slash, navigation, and non-ASCII source characters are never typo-simulated.',
                              '代码模式只会在安全的 ASCII 子集中产生临时错字；括号、引号、斜杠、导航动作和非 ASCII 源字符不会参与错字模拟。',
                            )
                          : l10n.typoWarning,
                      style: TextStyle(
                        color: Theme.of(context)
                            .colorScheme
                            .onSecondaryContainer,
                      ),
                    ),
                  ),
                ],
              ],
            ),
          ),
        ],
        const SizedBox(height: 16),
        AutoSaveFooter(
          controller: widget.controller,
          onReset: _restoreDefaults,
        ),
      ],
    );
  }

  String _speedSummary(ClipTypeLocalizations l10n) {
    final cps = _draft.charactersPerSecond.clamp(1, 250);
    final seconds = 100 / cps;
    final pace = cps <= 25
        ? l10n.text('steady', '稳定')
        : cps <= 70
        ? l10n.text('natural', '自然')
        : l10n.text('fast', '快速');
    return l10n.text(
      'About ${seconds.toStringAsFixed(seconds >= 10 ? 0 : 1)}s per 100 actions · $pace',
      '约 ${seconds.toStringAsFixed(seconds >= 10 ? 0 : 1)} 秒 / 100 个动作 · $pace',
    );
  }

  void _onCpsChanged(String value) {
    final parsed = int.tryParse(value);
    setState(() {
      _cpsError = value.isEmpty
          ? null
          : parsed == null || parsed < 1 || parsed > 250
          ? context.l10n.validationMessage('characters_per_second')
          : null;
    });
    if (parsed != null && parsed >= 1 && parsed <= 250) {
      _updateDraft(_draft.copyWith(charactersPerSecond: parsed));
    }
  }

  void _onThresholdChanged(String value) {
    final parsed = int.tryParse(value);
    setState(() {
      _thresholdError = value.isEmpty
          ? null
          : parsed == null || parsed < 1
          ? context.l10n.validationMessage('auto_clipboard_threshold')
          : null;
    });
    if (parsed != null && parsed >= 1) {
      _updateDraft(_draft.copyWith(autoClipboardThreshold: parsed));
    }
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
    setState(() {
      _cpsError = null;
      _thresholdError = null;
    });
  }
}

class _ModeCard extends StatelessWidget {
  const _ModeCard({
    required this.mode,
    required this.selected,
    required this.onTap,
  });

  final InjectionMode mode;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final (icon, title, description) = switch (mode) {
      InjectionMode.auto => (
        Icons.auto_awesome_outlined,
        l10n.text('Auto · Recommended', '自动选择 · 推荐'),
        l10n.text(
          'Chooses one safe backend for the session.',
          '为本次会话选择一种安全的输入方式。',
        ),
      ),
      InjectionMode.keyboard => (
        Icons.keyboard_alt_outlined,
        l10n.text('Type character by character', '逐字输入'),
        l10n.text('Adjustable pacing and corrected typos.', '可调输入节奏与纠错错字。'),
      ),
      InjectionMode.clipboard => (
        Icons.content_paste_go_outlined,
        l10n.text('Fast paste', '快速粘贴'),
        l10n.text(
          'One revision-guarded ordinary paste action.',
          '一次带剪贴板版本保护的普通粘贴。',
        ),
      ),
      InjectionMode.code => (
        Icons.code_rounded,
        l10n.text('Code input', '代码输入'),
        l10n.text(
          'Keyboard-only, auto-pair/indent aware.',
          '仅键盘输入，并与自动配对/缩进协作。',
        ),
      ),
    };
    final scheme = Theme.of(context).colorScheme;
    return Semantics(
      button: true,
      selected: selected,
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(14),
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 140),
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            color: selected
                ? scheme.secondaryContainer
                : scheme.surfaceContainerHighest,
            borderRadius: BorderRadius.circular(14),
            border: Border.all(
              color: selected ? scheme.primary : scheme.outlineVariant,
              width: selected ? 2 : 1,
            ),
          ),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(
                icon,
                color: selected ? scheme.primary : scheme.onSurfaceVariant,
              ),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      title,
                      style: Theme.of(context).textTheme.titleSmall
                          ?.copyWith(fontWeight: FontWeight.w700),
                    ),
                    const SizedBox(height: 4),
                    Text(
                      description,
                      style: Theme.of(context).textTheme.bodySmall
                          ?.copyWith(color: scheme.onSurfaceVariant),
                    ),
                  ],
                ),
              ),
              if (selected)
                Icon(Icons.check_circle, size: 19, color: scheme.primary),
            ],
          ),
        ),
      ),
    );
  }
}

class _InfoPanel extends StatelessWidget {
  const _InfoPanel({required this.icon, required this.text});

  final IconData icon;
  final String text;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(14),
      decoration: BoxDecoration(
        color: Theme.of(context).colorScheme.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Icon(icon, size: 20, color: Theme.of(context).colorScheme.primary),
          const SizedBox(width: 11),
          Expanded(child: Text(text)),
        ],
      ),
    );
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
