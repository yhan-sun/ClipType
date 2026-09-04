import 'package:flutter/material.dart';

import '../model/app_settings.dart';
import '../model/app_status.dart';

enum ClipTypeLanguage { english, simplifiedChinese }

extension ClipTypeLanguageValues on ClipTypeLanguage {
  Locale get locale => switch (this) {
    ClipTypeLanguage.english => const Locale('en'),
    ClipTypeLanguage.simplifiedChinese => const Locale('zh'),
  };

  String get code => switch (this) {
    ClipTypeLanguage.english => 'en',
    ClipTypeLanguage.simplifiedChinese => 'zh',
  };

  String get label => switch (this) {
    ClipTypeLanguage.english => 'English',
    ClipTypeLanguage.simplifiedChinese => '简体中文',
  };

  static ClipTypeLanguage fromCode(String? code) {
    return code == 'zh'
        ? ClipTypeLanguage.simplifiedChinese
        : ClipTypeLanguage.english;
  }
}

class ClipTypeLocalizations {
  ClipTypeLocalizations(this.locale);

  final Locale locale;

  bool get isChinese => locale.languageCode == 'zh';

  static const delegate = _ClipTypeLocalizationsDelegate();
  static const supportedLocales = [Locale('en'), Locale('zh')];

  String _text(String english, String chinese) => isChinese ? chinese : english;

  String get general => _text('General', '常规');
  String get shortcuts => _text('Shortcuts', '快捷键');
  String get typing => _text('Typing', '输入');
  String get permissions => _text('Permissions', '权限');
  String get about => _text('About', '关于');
  String get trigger => _text('Trigger', '触发');
  String get cancel => _text('Cancel', '取消');
  String get ready => _text('Ready', '就绪');
  String get changesSaved => _text('Saved', '已保存');
  String get changesPending => _text('Pending', '待保存');
  String get savingChanges => _text('Saving…', '保存中…');
  String get saveFailed => _text('Save failed', '保存失败');
  String get reviewChanges => _text('Review changes', '请检查修改');
  String get retrySave => _text('Retry save', '重试保存');
  String get autoSaveHint => _text(
    'Changes save automatically. You can keep working.',
    '修改会自动保存，可以继续操作。',
  );
  String get restoreDefaults => _text('Restore page defaults', '恢复本页默认');
  String get sessionStarted => _text('Typing session started.', '输入会话已开始。');
  String get loading => _text('Loading ClipType…', '正在加载 ClipType…');
  String get interfaceLanguage => _text('Interface language', '界面语言');
  String get interfaceLanguageSubtitle => _text(
    'Choose the language used by the ClipType settings window and menu.',
    '选择 ClipType 设置窗口和菜单使用的语言。',
  );
  String get languageEnglish => 'English';
  String get languageChinese => '简体中文';

  String get generalDescription => _text(
    'ClipType reads the current clipboard only after an explicit trigger and delivers it to the destination that was active at that moment.',
    'ClipType 仅在明确触发后读取当前剪贴板，并将其发送到触发时处于活动状态的目标。',
  );
  String get application => _text('Application', '应用');
  String get futureSessions => _text(
    'These switches affect future sessions immediately.',
    '这些开关会立即影响后续会话。',
  );
  String get enableClipType => _text('Enable ClipType', '启用 ClipType');
  String get enableClipTypeSubtitle =>
      _text('Accept the configured global Trigger shortcut.', '接受配置的全局触发快捷键。');
  String get notifications => _text('Notifications', '通知');
  String get notificationsSubtitle => _text(
    'Show fixed, content-free status notifications.',
    '显示固定且不含内容的状态通知。',
  );
  String get startAtLogin => _text('Start at Login', '登录时启动');
  String get startAtLoginUnsupported => _text(
    'The current macOS version does not provide a supported login item.',
    '当前 macOS 版本不提供受支持的登录项。',
  );
  String get startAtLoginSubtitle => _text(
    'Use the app-owned macOS login item; no administrator permission is used.',
    '使用由应用管理的 macOS 登录项；不需要管理员权限。',
  );
  String get currentRuntimeStatus => _text('Current runtime status', '当前运行状态');
  String get activeSessionStatus => _text(
    'A single bounded session is active. Settings changes do not alter its immutable snapshot.',
    '一个有界会话正在运行。设置变化不会修改该会话的不可变快照。',
  );
  String get readyForTrigger =>
      _text('Ready for an explicit Trigger.', '等待明确触发。');
  String get lastSessionCompleted =>
      _text('The last session completed.', '上一次会话已完成。');
  String get lastSessionCancelled =>
      _text('The last session was cancelled safely.', '上一次会话已安全取消。');
  String get lastSessionTargetChanged => _text(
    'The last session stopped because the destination changed.',
    '上一次会话因目标发生变化而停止。',
  );
  String get lastSessionClipboardChanged => _text(
    'The last session stopped because the clipboard revision changed.',
    '上一次会话因剪贴板版本发生变化而停止。',
  );
  String get lastSessionStopped =>
      _text('The last session stopped safely.', '上一次会话已安全停止。');
  String get backendPrefix => _text('Backend', '后端');
  String get cancelActiveSession => _text('Cancel active session', '取消活动会话');

  String get shortcutsDescription => _text(
    'Record a complete modifier-plus-key pair while the recorder owns local focus. macOS registration is probed before the pair is committed.',
    '在录制器拥有本地焦点时录制完整的修饰键加按键组合。提交前会先探测 macOS 注册结果。',
  );
  String get globalCommands => _text('Global commands', '全局命令');
  String get globalCommandsSubtitle => _text(
    'Modifier-only presses do not complete a recording. Escape cancels; Delete or Backspace clears the candidate.',
    '单独按修饰键不会完成录制。Escape 取消；Delete 或 Backspace 清空候选组合。',
  );
  String get triggerShortcut => _text('Trigger shortcut', '触发快捷键');
  String get cancelShortcut => _text('Cancel shortcut', '取消快捷键');
  String get osAvailability => _text('OS-level availability', '系统级可用性');
  String get osAvailabilitySubtitle => _text(
    'An Available result means macOS accepted temporary global registrations. Application-local conflicts and hook-based tools cannot be fully verified.',
    'Available 表示 macOS 接受了临时全局注册。应用内冲突和基于 Hook 的工具无法完全验证。',
  );
  String get checkAvailability => _text('Check availability', '检查可用性');
  String get bothShortcutsRequired =>
      _text('Both shortcuts are required.', '必须设置两个快捷键。');
  String get shortcutsMustDiffer =>
      _text('Trigger and Cancel must be different.', '触发和取消快捷键必须不同。');

  String get typingDescription => _text(
    'Rust freezes one backend and one immutable settings snapshot per session. Explicit Keyboard, Clipboard, and Code choices never silently fall back.',
    'Rust 会为每个会话冻结一个后端和一份不可变设置快照。明确选择 Keyboard、Clipboard 或 Code 时不会静默切换。',
  );
  String get deliveryMode => _text('Delivery mode', '传输模式');
  String get mode => _text('Mode', '模式');
  String get humanPacedControls =>
      _text('Human-paced keyboard controls', '模拟键入控制');
  String get keyboardControlsSubtitle => _text(
    'These controls affect Keyboard and Code pacing. Clipboard remains one bounded Command+V action.',
    '这些控制项影响 Keyboard 和 Code 的键盘节奏。Clipboard 仍是一次有界的 Command+V 操作。',
  );
  String get codeModeSubtitle => _text(
    'Code mode uses keyboard actions: it skips leading indentation, types opening delimiters and ordinary quotes, and moves right over matching closers already generated by the editor. Triple-quoted strings (""" or \'\'\') and Markdown triple-backtick fences are typed explicitly. Content inside strings and comments is kept; typo simulation is disabled. Editor auto-pair must be enabled for ordinary pairs.',
    'Code 模式走键盘事件：跳过每行行首缩进，输入开括号和普通引号，并用右方向键跳过编辑器已生成的对应闭括号或引号。三引号字符串（""" 或 \'\'\'）和 Markdown 三反引号围栏会显式输入。字符串和注释中的内容会保留，且不使用错字模拟；普通配对需要开启编辑器自动补全。',
  );
  String get charactersPerSecond => _text('Characters per second', '每秒字符数');
  String get charactersPerSecondHelper =>
      _text('1–250 actions per second', '每秒 1–250 个动作');
  String get timingJitter => _text('Timing jitter', '时间抖动');
  String get correctedTypoProbability =>
      _text('Corrected typo probability', '纠错错字概率');
  String get typoWarning => _text(
    'Do not use corrected typo simulation for passwords, source code, terminals, commands, checksums, or exact-data entry.',
    '不要在密码、源代码、终端、命令、校验和或精确数据录入中使用纠错错字模拟。',
  );
  String get autoPolicy => _text('Auto policy', '自动策略');
  String get autoPolicySubtitle => _text(
    'Auto prefers the revision-guarded current pasteboard for non-ASCII, complex, or large text when macOS capabilities are available.',
    '当 macOS 能力可用时，Auto 会优先对非 ASCII、复杂或大型文本使用带版本保护的当前剪贴板。',
  );
  String get autoClipboardThreshold =>
      _text('Auto clipboard threshold', '自动剪贴板阈值');
  String get semanticElementsMinimum =>
      _text('Semantic elements; minimum 1', '语义元素数；最小值为 1');

  String get permissionsDescription => _text(
    'ClipType asks macOS for Accessibility only after you explicitly request it. The app never bypasses consent and fails closed when trust is absent or revoked.',
    'ClipType 仅在你明确请求后向 macOS 请求辅助功能权限。应用不会绕过授权；未获得或被撤销信任时会安全停止。',
  );
  String get accessibility => _text('Accessibility', '辅助功能');
  String get syntheticInputAvailable => _text(
    'Synthetic input and focus evidence are available to the native adapter.',
    '原生适配器可以使用模拟输入和焦点证据。',
  );
  String get grantAccess => _text(
    'Grant ClipType access in System Settings to enable cross-application input. No clipboard text is shown here.',
    '请在系统设置中授予 ClipType 辅助功能权限，以启用跨应用输入。此处不会显示剪贴板文本。',
  );
  String get requestPermission => _text('Request Permission', '请求权限');
  String get openSystemSettings => _text('Open System Settings', '打开系统设置');
  String get safetyBoundary => _text('Safety boundary', '安全边界');
  String get safetyBoundaryText => _text(
    'ClipType captures only content-free destination identity and permission state. It does not read focused-field values, window titles, selected text, or arbitrary keyboard history.',
    'ClipType 只记录不含内容的目标身份和权限状态。它不会读取焦点字段内容、窗口标题、选中文本或任意键盘历史。',
  );

  String get aboutTitle => _text('About ClipType', '关于 ClipType');
  String get aboutDescription => _text(
    'A local, privacy-first clipboard-to-input utility for one explicit user action.',
    '一个本地、隐私优先、由用户明确触发的剪贴板到输入工具。',
  );
  String get buildInformation => _text('Build information', '构建信息');
  String get version => _text('Version', '版本');
  String get architecture => _text('Architecture', '架构');
  String get ui => _text('UI', '界面');
  String get nativeShell => _text('Native shell', '原生壳');
  String get releaseStatus => _text('Release status', '发布状态');
  String get unsignedLocalCandidate =>
      _text('Unsigned local candidate', '未签名本地候选版本');
  String get project => _text('Project', '项目');
  String get sourceRepository => _text('Source repository', '源代码仓库');
  String get license => _text('License', '许可证');
  String get privacyPromise => _text('Privacy promise', '隐私承诺');
  String get privacyPromiseText => _text(
    'Clipboard and injected text stay in memory for the active bounded session. The bridge, event channel, settings, and diagnostics carry only configuration, counts, identities, and fixed outcome categories.',
    '剪贴板文本和注入文本只在活动的有界会话期间保留于内存中。Bridge、事件通道、设置和诊断信息只传递配置、计数、身份及固定结果类别。',
  );

  String get record => _text('Record', '录制');
  String get clear => _text('Clear', '清空');
  String get reset => _text('Reset', '重置');
  String get recordPrompt => _text('Press a modifier + key…', '请按下修饰键加按键组合…');
  String get escapeToCancel => _text('Esc to cancel', 'Esc 取消');
  String get notSet => _text('Not set', '未设置');

  String modeLabel(InjectionMode mode) => switch (mode) {
    InjectionMode.keyboard => _text('Keyboard', '键盘'),
    InjectionMode.clipboard => _text('Clipboard', '剪贴板'),
    InjectionMode.auto => _text('Auto', '自动'),
    InjectionMode.code => _text('Code', '代码'),
  };

  String phaseLabel(SessionPhase phase) => switch (phase) {
    SessionPhase.idle => ready,
    SessionPhase.preparing => _text('Preparing', '准备中'),
    SessionPhase.injecting => _text('Typing', '输入中'),
    SessionPhase.cancelling => _text('Cancelling', '取消中'),
  };

  String backendLabel(String? backend) => switch (backend) {
    'keyboard' => modeLabel(InjectionMode.keyboard),
    'clipboard' => modeLabel(InjectionMode.clipboard),
    'code' => modeLabel(InjectionMode.code),
    _ => _text('Unknown', '未知'),
  };

  String permissionLabel(String state) => switch (state) {
    'granted' => _text('Granted', '已授权'),
    'revoked' => _text('Revoked', '已撤销'),
    'not_requested' => _text('Not requested', '未请求'),
    'not_granted' => _text('Not granted', '未授权'),
    'not_required' => _text('Not required', '不需要'),
    _ => _text('Unknown', '未知'),
  };

  String startupLabel(String state) => switch (state) {
    'enabled' => _text('Enabled', '已启用'),
    'requires_approval' => _text('Needs approval', '需要批准'),
    'not_registered' => _text('Off', '关闭'),
    'unsupported' => _text('Unsupported', '不支持'),
    _ => _text('Unknown', '未知'),
  };

  String availabilityLabel(String value) => switch (value) {
    'available' => _text('Available', '可用'),
    'conflict' => _text('Conflict', '冲突'),
    'reserved' => _text('Reserved', '已保留'),
    'unsupported' => _text('Unsupported', '不支持'),
    _ => _text('Not checked', '未检查'),
  };

  String completionLabel(String value) => switch (value) {
    'completed' => lastSessionCompleted,
    'cancelled' => lastSessionCancelled,
    'target_changed' => lastSessionTargetChanged,
    'clipboard_changed' => lastSessionClipboardChanged,
    _ => lastSessionStopped,
  };

  String validationMessage(String? code) => switch (code) {
    'missing_hotkeys' || 'shortcuts_required' => _text(
      'Record both a Trigger and Cancel shortcut before saving.',
      '保存前请录制触发和取消两个快捷键。',
    ),
    'different_hotkeys' => _text(
      'Trigger and Cancel shortcuts must be different.',
      '触发和取消快捷键必须不同。',
    ),
    'characters_per_second' => _text(
      'Characters per second must be between 1 and 250.',
      '每秒字符数必须在 1 到 250 之间。',
    ),
    'jitter_percent' => _text(
      'Jitter must be between 0% and 95%.',
      '抖动必须在 0% 到 95% 之间。',
    ),
    'typo_probability_percent' => _text(
      'Corrected typo probability must be between 0% and 25%.',
      '纠错错字概率必须在 0% 到 25% 之间。',
    ),
    'auto_clipboard_threshold' => _text(
      'The Auto clipboard threshold must be at least 1.',
      '自动剪贴板阈值必须至少为 1。',
    ),
    _ => _text('The settings are invalid.', '设置无效。'),
  };

  String resultMessage(String? result) => switch (result) {
    'ok' => _text('Ready.', '已就绪。'),
    'started' => _text('Typing session started.', '输入会话已开始。'),
    'busy' => _text('Another typing session is already active.', '已有输入会话正在运行。'),
    'cancel_requested' => _text('Cancelling the active session…', '正在取消活动会话…'),
    'idle' => _text('No active session.', '当前没有活动会话。'),
    'rejected' => _text(
      'The operation was rejected by a safety check.',
      '安全检查拒绝了该操作。',
    ),
    'permission_required' => _text(
      'Accessibility permission is missing. Turn on ClipType in System Settings > Privacy & Security > Accessibility, then try again.',
      '尚未获得辅助功能权限。请在“系统设置 > 隐私与安全性 > 辅助功能”中开启 ClipType，然后重试。',
    ),
    'prompt_requested' => _text(
      'System Settings is ready. Turn on ClipType under Privacy & Security > Accessibility, then try again.',
      '系统设置已打开。请在“隐私与安全性 > 辅助功能”中开启 ClipType，然后重试。',
    ),
    'settings_opened' => _text(
      'System Settings opened. Turn on ClipType under Privacy & Security > Accessibility, then try again.',
      '系统设置已打开。请在“隐私与安全性 > 辅助功能”中开启 ClipType，然后重试。',
    ),
    'already_granted' => _text(
      'Accessibility permission is already granted.',
      '辅助功能权限已授权。',
    ),
    'conflict' => _text(
      'The shortcut is already registered globally by another app.',
      '快捷键已被其他应用全局注册。',
    ),
    'reserved' => _text(
      'That shortcut is reserved by macOS.',
      '该快捷键被 macOS 保留。',
    ),
    'unsupported' => _text(
      'That shortcut is not supported on macOS.',
      '该快捷键不受 macOS 支持。',
    ),
    'native_failure' => _text(
      'The native operation failed safely.',
      '原生操作已安全失败。',
    ),
    _ => _text('The operation finished safely.', '操作已安全结束。'),
  };

  String availabilityMessage(String availability) => switch (availability) {
    'available' => _text(
      'macOS accepted both shortcuts. App-local conflicts cannot be fully verified.',
      'macOS 已接受两个快捷键。应用内冲突无法完全验证。',
    ),
    'conflict' => _text(
      'One or both shortcuts are already registered globally.',
      '一个或两个快捷键已被全局注册。',
    ),
    'reserved' => _text(
      'One or both shortcuts are reserved by macOS.',
      '一个或两个快捷键被 macOS 保留。',
    ),
    'unsupported' => _text(
      'One or both shortcuts are unsupported on macOS.',
      '一个或两个快捷键不受 macOS 支持。',
    ),
    _ => _text(
      'Shortcut availability cannot be fully verified.',
      '无法完全验证快捷键可用性。',
    ),
  };

  String completionMessage(String? completion) => switch (completion) {
    'completed' => _text('Typing completed.', '输入已完成。'),
    'cancelled' => _text('Typing cancelled.', '输入已取消。'),
    'target_changed' => _text(
      'Destination changed; remaining input was stopped.',
      '目标发生变化；剩余输入已停止。',
    ),
    'clipboard_changed' => _text(
      'Clipboard changed before paste; nothing further was sent.',
      '粘贴前剪贴板发生变化；未继续发送内容。',
    ),
    _ => _text('The input operation stopped safely.', '输入操作已安全停止。'),
  };

  String errorMessage(String code) => switch (code) {
    'bridge_unavailable' => _text(
      'The native ClipType bridge is unavailable.',
      'ClipType 原生 Bridge 不可用。',
    ),
    'invalid_configuration' => _text(
      'The saved ClipType settings are invalid.',
      '已保存的 ClipType 设置无效。',
    ),
    'settings_failed' => _text('Settings could not be saved.', '设置无法保存。'),
    'trigger_failed' => _text(
      'The trigger command could not be delivered.',
      '无法传递触发命令。',
    ),
    'cancel_failed' => _text(
      'The cancel command could not be delivered.',
      '无法传递取消命令。',
    ),
    'permission_request_failed' => _text(
      'The Accessibility request could not be opened.',
      '无法打开辅助功能权限请求。',
    ),
    'system_settings_failed' => _text(
      'System Settings could not be opened.',
      '无法打开系统设置。',
    ),
    'availability_failed' => _text(
      'Shortcut availability could not be checked.',
      '无法检查快捷键可用性。',
    ),
    _ => _text(
      'The native ClipType operation failed safely.',
      'ClipType 原生操作已安全失败。',
    ),
  };
}

extension ClipTypeLocalizationsContext on BuildContext {
  ClipTypeLocalizations get l10n =>
      Localizations.of<ClipTypeLocalizations>(this, ClipTypeLocalizations) ??
      ClipTypeLocalizations(const Locale('en'));
}

class _ClipTypeLocalizationsDelegate
    extends LocalizationsDelegate<ClipTypeLocalizations> {
  const _ClipTypeLocalizationsDelegate();

  @override
  bool isSupported(Locale locale) =>
      locale.languageCode == 'en' || locale.languageCode == 'zh';

  @override
  Future<ClipTypeLocalizations> load(Locale locale) async =>
      ClipTypeLocalizations(locale);

  @override
  bool shouldReload(_ClipTypeLocalizationsDelegate old) => false;
}
