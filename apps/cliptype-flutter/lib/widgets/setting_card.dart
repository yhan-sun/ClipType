import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';

class SettingCard extends StatelessWidget {
  const SettingCard({
    required this.child,
    this.title,
    this.subtitle,
    super.key,
  });

  final String? title;
  final String? subtitle;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Card(
      margin: EdgeInsets.zero,
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            if (title != null) ...[
              Text(title!, style: theme.textTheme.titleMedium),
              if (subtitle != null) ...[
                const SizedBox(height: 5),
                Text(
                  subtitle!,
                  style: theme.textTheme.bodyMedium?.copyWith(
                    color: theme.colorScheme.onSurfaceVariant,
                  ),
                ),
              ],
              const SizedBox(height: 16),
            ],
            child,
          ],
        ),
      ),
    );
  }
}

class PageHeader extends StatelessWidget {
  const PageHeader({required this.title, required this.description, super.key});

  final String title;
  final String description;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title, style: theme.textTheme.headlineMedium),
        const SizedBox(height: 8),
        ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 700),
          child: Text(
            description,
            style: theme.textTheme.bodyLarge?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
              height: 1.45,
            ),
          ),
        ),
      ],
    );
  }
}

class StatusPill extends StatelessWidget {
  const StatusPill({required this.label, this.good = false, super.key});

  final String label;
  final bool good;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final color = good ? theme.colorScheme.primary : theme.colorScheme.outline;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.12),
        borderRadius: BorderRadius.circular(100),
        border: Border.all(color: color.withValues(alpha: 0.35)),
      ),
      child: Text(
        label,
        style: theme.textTheme.labelMedium?.copyWith(
          color: color,
          fontWeight: FontWeight.w700,
        ),
      ),
    );
  }
}

class SaveBar extends StatelessWidget {
  const SaveBar({
    required this.onApply,
    required this.onReset,
    required this.saving,
    this.error,
    super.key,
  });

  final VoidCallback onApply;
  final VoidCallback onReset;
  final bool saving;
  final String? error;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = context.l10n;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        if (error != null) ...[
          Text(
            error!,
            style: theme.textTheme.bodyMedium?.copyWith(
              color: theme.colorScheme.error,
            ),
          ),
          const SizedBox(height: 12),
        ],
        Row(
          mainAxisAlignment: MainAxisAlignment.end,
          children: [
            TextButton(
              onPressed: saving ? null : onReset,
              child: Text(l10n.reset),
            ),
            const SizedBox(width: 10),
            FilledButton.icon(
              onPressed: saving ? null : onApply,
              icon: saving
                  ? const SizedBox(
                      width: 16,
                      height: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : const Icon(Icons.check),
              label: Text(l10n.apply),
            ),
          ],
        ),
      ],
    );
  }
}

class InfoRow extends StatelessWidget {
  const InfoRow({required this.label, required this.value, super.key});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 180,
            child: Text(
              label,
              style: theme.textTheme.labelLarge?.copyWith(
                color: theme.colorScheme.onSurfaceVariant,
              ),
            ),
          ),
          Expanded(child: Text(value)),
        ],
      ),
    );
  }
}
