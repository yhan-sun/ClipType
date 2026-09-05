import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../state/settings_controller.dart';

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
      clipBehavior: Clip.antiAlias,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(18, 16, 18, 18),
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

class PageContent extends StatelessWidget {
  const PageContent({required this.children, super.key});

  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final horizontal = constraints.maxWidth < 948
            ? 24.0
            : (constraints.maxWidth - 900) / 2;
        return ListView(
          padding: EdgeInsets.fromLTRB(horizontal, 28, horizontal, 32),
          children: [
            Align(
              alignment: Alignment.topCenter,
              child: ConstrainedBox(
                constraints: const BoxConstraints(maxWidth: 760),
                child: SizedBox(
                  width: double.infinity,
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: children,
                  ),
                ),
              ),
            ),
          ],
        );
      },
    );
  }
}

class PageHeader extends StatelessWidget {
  const PageHeader({
    required this.title,
    required this.description,
    this.trailing,
    super.key,
  });

  final String title;
  final String description;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final copy = Column(
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
    if (trailing == null) return copy;

    return LayoutBuilder(
      builder: (context, constraints) {
        if (constraints.maxWidth < 620) {
          return Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [copy, const SizedBox(height: 14), trailing!],
          );
        }
        return Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Expanded(child: copy),
            const SizedBox(width: 18),
            trailing!,
          ],
        );
      },
    );
  }
}

enum StatusTone { neutral, success, active, warning, error, disabled }

class StatusPill extends StatelessWidget {
  const StatusPill({
    required this.label,
    this.good = false,
    this.tone,
    super.key,
  });

  final String label;
  final bool good;
  final StatusTone? tone;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final resolved = tone ?? (good ? StatusTone.success : StatusTone.neutral);
    final color = switch (resolved) {
      StatusTone.success => theme.colorScheme.primary,
      StatusTone.active => theme.colorScheme.tertiary,
      StatusTone.warning => theme.colorScheme.secondary,
      StatusTone.error => theme.colorScheme.error,
      StatusTone.disabled => theme.colorScheme.outline,
      StatusTone.neutral => theme.colorScheme.onSurfaceVariant,
    };
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

class AutoSaveIndicator extends StatelessWidget {
  const AutoSaveIndicator({required this.controller, super.key});

  final SettingsController controller;

  @override
  Widget build(BuildContext context) {
    if (controller.autoSaveStatus == AutoSaveStatus.saved &&
        controller.validationError == null) {
      return const SizedBox.shrink();
    }
    final theme = Theme.of(context);
    final l10n = context.l10n;
    final (icon, label, color) = switch (controller.autoSaveStatus) {
      AutoSaveStatus.saved => (
        Icons.check_circle_outline,
        l10n.changesSaved,
        theme.colorScheme.primary,
      ),
      AutoSaveStatus.pending => (
        Icons.schedule_outlined,
        l10n.changesPending,
        theme.colorScheme.secondary,
      ),
      AutoSaveStatus.saving => (
        Icons.sync,
        l10n.savingChanges,
        theme.colorScheme.primary,
      ),
      AutoSaveStatus.error => (
        controller.validationError == null
            ? Icons.error_outline
            : Icons.warning_amber_outlined,
        controller.validationError == null
            ? l10n.saveFailed
            : l10n.reviewChanges,
        theme.colorScheme.error,
      ),
    };
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 11, vertical: 8),
      decoration: BoxDecoration(
        color: color.withValues(alpha: 0.10),
        borderRadius: BorderRadius.circular(100),
        border: Border.all(color: color.withValues(alpha: 0.28)),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          if (controller.autoSaveStatus == AutoSaveStatus.saving)
            SizedBox(
              width: 15,
              height: 15,
              child: CircularProgressIndicator(strokeWidth: 2, color: color),
            )
          else
            Icon(icon, size: 17, color: color),
          const SizedBox(width: 7),
          Text(
            label,
            style: theme.textTheme.labelLarge?.copyWith(
              color: color,
              fontWeight: FontWeight.w700,
            ),
          ),
        ],
      ),
    );
  }
}

class AutoSaveFooter extends StatelessWidget {
  const AutoSaveFooter({
    required this.controller,
    required this.onReset,
    super.key,
  });

  final SettingsController controller;
  final VoidCallback onReset;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = context.l10n;
    return Container(
      width: double.infinity,
      padding: const EdgeInsets.fromLTRB(14, 12, 10, 12),
      decoration: BoxDecoration(
        color: theme.colorScheme.surfaceContainerHighest.withValues(
          alpha: 0.55,
        ),
        borderRadius: BorderRadius.circular(14),
      ),
      child: LayoutBuilder(
        builder: (context, constraints) {
          final details = Row(
            children: [
              Icon(
                Icons.save_outlined,
                size: 19,
                color: theme.colorScheme.onSurfaceVariant,
              ),
              const SizedBox(width: 9),
              Flexible(child: Text(l10n.autoSaveHint)),
            ],
          );
          final retry = controller.canRetrySave
              ? TextButton.icon(
                  onPressed: controller.saving
                      ? null
                      : controller.retryFailedSave,
                  icon: const Icon(Icons.refresh, size: 18),
                  label: Text(l10n.retrySave),
                )
              : null;
          final reset = TextButton.icon(
            onPressed: controller.saving ? null : onReset,
            icon: const Icon(Icons.restart_alt, size: 18),
            label: Text(l10n.restoreDefaults),
          );
          if (constraints.maxWidth < 560) {
            return Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                details,
                const SizedBox(height: 8),
                Wrap(
                  alignment: WrapAlignment.end,
                  crossAxisAlignment: WrapCrossAlignment.center,
                  spacing: 4,
                  runSpacing: 4,
                  children: [
                    AutoSaveIndicator(controller: controller),
                    retry ?? const SizedBox.shrink(),
                    reset,
                  ],
                ),
              ],
            );
          }
          return Row(
            children: [
              Expanded(child: details),
              AutoSaveIndicator(controller: controller),
              const SizedBox(width: 8),
              retry ?? const SizedBox.shrink(),
              const SizedBox(width: 4),
              reset,
            ],
          );
        },
      ),
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
    final labelWidget = Text(
      label,
      style: theme.textTheme.labelLarge?.copyWith(
        color: theme.colorScheme.onSurfaceVariant,
      ),
    );
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: LayoutBuilder(
        builder: (context, constraints) {
          if (constraints.maxWidth < 480) {
            return Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                labelWidget,
                const SizedBox(height: 4),
                SelectableText(value),
              ],
            );
          }
          return Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              SizedBox(width: 160, child: labelWidget),
              Expanded(child: SelectableText(value)),
            ],
          );
        },
      ),
    );
  }
}
