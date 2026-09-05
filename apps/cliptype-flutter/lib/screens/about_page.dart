import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../state/settings_controller.dart';
import '../widgets/setting_card.dart';

class AboutPage extends StatelessWidget {
  const AboutPage({required this.controller, super.key});

  final SettingsController controller;

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    final build = controller.buildInfo;
    final diagnostics = build.diagnostics(
      permission: controller.status.permission,
      phase: controller.status.phase.name,
      bridgeAvailable: controller.status.bridgeAvailable,
    );
    return PageContent(
      children: [
        PageHeader(
          title: l10n.aboutTitle,
          description: l10n.text(
            'Build identity and privacy-safe diagnostics for the ClipType instance you are actually running.',
            '查看当前正在运行的 ClipType 构建身份，以及不包含剪贴板/目标内容的安全诊断信息。',
          ),
        ),
        const SizedBox(height: 24),
        SettingCard(
          title: l10n.buildInformation,
          child: Column(
            children: [
              InfoRow(
                label: l10n.version,
                value: '${build.version} (${build.build})',
              ),
              InfoRow(label: l10n.architecture, value: build.architecture),
              InfoRow(
                label: l10n.text('Release channel', '发布通道'),
                value: build.channel,
              ),
              InfoRow(label: l10n.text('Signing', '签名'), value: build.signing),
              InfoRow(
                label: l10n.text('Notarization', '公证'),
                value: build.notarization,
              ),
              InfoRow(label: l10n.ui, value: 'Flutter macOS desktop'),
              InfoRow(label: l10n.nativeShell, value: 'Swift / AppKit + Rust'),
            ],
          ),
        ),
        const SizedBox(height: 16),
        SettingCard(
          title: l10n.text('Diagnostics', '诊断信息'),
          subtitle: l10n.text(
            'This block contains only build and fixed runtime categories. It never includes clipboard text, destination text, selected text, document names, or window titles.',
            '这里只包含构建信息和固定运行状态类别，不包含剪贴板文本、目标文本、选中文本、文档名称或窗口标题。',
          ),
          child: Container(
            width: double.infinity,
            padding: const EdgeInsets.all(14),
            decoration: BoxDecoration(
              color: Theme.of(context).colorScheme.surfaceContainerHighest,
              borderRadius: BorderRadius.circular(10),
            ),
            child: SelectableText(
              diagnostics,
              style: Theme.of(context).textTheme.bodySmall
                  ?.copyWith(fontFamily: 'monospace', height: 1.55),
            ),
          ),
        ),
        const SizedBox(height: 16),
        SettingCard(
          title: l10n.project,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(l10n.sourceRepository),
              const SizedBox(height: 6),
              const SelectableText('https://github.com/yhan-sun/ClipType'),
              const SizedBox(height: 16),
              Text(l10n.license),
              const SizedBox(height: 6),
              const Text('MIT OR Apache-2.0'),
            ],
          ),
        ),
        const SizedBox(height: 16),
        SettingCard(
          title: l10n.privacyPromise,
          child: Text(l10n.privacyPromiseText),
        ),
      ],
    );
  }
}
