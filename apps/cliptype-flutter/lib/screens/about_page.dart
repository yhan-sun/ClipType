import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../widgets/setting_card.dart';

class AboutPage extends StatelessWidget {
  const AboutPage({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = context.l10n;
    return ListView(
      padding: const EdgeInsets.fromLTRB(32, 30, 32, 32),
      children: [
        PageHeader(title: l10n.aboutTitle, description: l10n.aboutDescription),
        SizedBox(height: 26),
        SettingCard(
          title: l10n.buildInformation,
          child: Column(
            children: [
              InfoRow(label: l10n.version, value: '0.2.0-beta.2'),
              InfoRow(label: l10n.architecture, value: 'Apple Silicon arm64'),
              InfoRow(label: l10n.ui, value: 'Flutter macOS desktop'),
              InfoRow(label: l10n.nativeShell, value: 'Swift / AppKit'),
              InfoRow(
                label: l10n.releaseStatus,
                value: l10n.unsignedLocalCandidate,
              ),
            ],
          ),
        ),
        SizedBox(height: 16),
        SettingCard(
          title: l10n.project,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(l10n.sourceRepository),
              SizedBox(height: 6),
              SelectableText('https://github.com/yhan-sun/ClipType'),
              SizedBox(height: 16),
              Text(l10n.license),
              SizedBox(height: 6),
              Text('MIT OR Apache-2.0'),
            ],
          ),
        ),
        SizedBox(height: 16),
        SettingCard(
          title: l10n.privacyPromise,
          child: Text(l10n.privacyPromiseText),
        ),
      ],
    );
  }
}
