//! Content-free macOS runtime probe for the P3-S01 physical validation gate.
//!
//! This executable is validation tooling. It never prints clipboard text,
//! focused-control content, window titles, native handles, or clipboard revision
//! values. Commands that mutate permission, login-item, or hotkey state require an
//! explicit operator invocation.

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("p3_s01 status=unsupported platform=macos_required");
    std::process::exit(2);
}

#[cfg(target_os = "macos")]
fn main() {
    if let Err(error) = macos_probe::run() {
        eprintln!("p3_s01 status=failed category={error}");
        std::process::exit(1);
    }
}

#[cfg(target_os = "macos")]
mod macos_probe {
    use std::{
        env,
        sync::mpsc,
        thread,
        time::{Duration, Instant},
    };

    use cliptype_core::{
        HotkeyApplyResult, HotkeyAvailability, HotkeyPair, HotkeySpec, InjectionMode,
        NativeByteLimit,
    };
    use cliptype_macos::{
        MacAccessibility, MacClipboard, MacHotkeyController, MacKeyboard, MacMenuEvent,
        MacModifiers, MacPaste, MacStartup, MacStatusItem, MacTarget, initialize_application,
    };
    use cliptype_platform::{
        AccessibilityPermissionPort, ClipboardPort, KeyboardPort, ModifierPort, PastePort,
        TargetPort,
    };

    const CLIPBOARD_LIMIT_BYTES: usize = 8 * 1024 * 1024;
    const DEFAULT_WATCH_SECONDS: u64 = 180;
    const MAX_WATCH_SECONDS: u64 = 1_800;

    pub fn run() -> Result<(), &'static str> {
        initialize_application();
        let mut arguments = env::args().skip(1);
        let command = arguments.next().unwrap_or_else(|| "snapshot".to_owned());
        match command.as_str() {
            "snapshot" => snapshot(),
            "permission-watch" => {
                let seconds = arguments
                    .next()
                    .map(|value| value.parse::<u64>().map_err(|_| "invalid_watch_seconds"))
                    .transpose()?
                    .unwrap_or(DEFAULT_WATCH_SECONDS)
                    .min(MAX_WATCH_SECONDS);
                permission_watch(seconds)
            }
            "open-permission-settings" => open_permission_settings(),
            "hotkey-cycle" => hotkey_cycle(),
            "hold-hotkeys" => {
                let trigger = arguments.next().ok_or("missing_trigger")?;
                let cancel = arguments.next().ok_or("missing_cancel")?;
                let seconds = arguments
                    .next()
                    .map(|value| value.parse::<u64>().map_err(|_| "invalid_hold_seconds"))
                    .transpose()?
                    .unwrap_or(DEFAULT_WATCH_SECONDS)
                    .min(MAX_WATCH_SECONDS);
                hold_hotkeys(&trigger, &cancel, seconds)
            }
            "status-item-smoke" => status_item_smoke(),
            "startup-status" => startup_status(),
            "startup-enable" => startup_set(true),
            "startup-disable" => startup_set(false),
            _ => Err("unknown_command"),
        }
    }

    fn snapshot() -> Result<(), &'static str> {
        let permission = MacAccessibility::default();
        println!("format=cliptype-p3-s01-v1");
        println!("command=snapshot");
        println!(
            "source_sha={}",
            option_env!("CLIPTYPE_SOURCE_SHA").unwrap_or("unknown")
        );
        println!("permission={}", permission_label(permission.state()));

        let keyboard = MacKeyboard.capabilities();
        println!(
            "keyboard_unicode={}",
            capability_label(keyboard.unicode_text)
        );
        println!(
            "keyboard_line_break={}",
            capability_label(keyboard.line_break)
        );
        println!("keyboard_tab={}", capability_label(keyboard.tab));
        println!(
            "modifier_observation_capability={}",
            capability_label(keyboard.modifier_observation)
        );

        let paste = MacPaste.capabilities();
        println!("paste_chord={}", capability_label(paste.paste_chord));
        println!(
            "paste_revision_guard={}",
            capability_label(paste.clipboard_revision_guard)
        );
        println!(
            "modifiers={}",
            modifier_label(MacModifiers.observe_modifiers())
        );

        match MacTarget.capture() {
            Ok(target) => {
                println!("target_capture=available");
                println!(
                    "target_process_id_present={}",
                    target.metadata().process_id.is_some()
                );
                println!("target_strength={}", evidence_label(target.strength()));
            }
            Err(error) => {
                println!("target_capture={}", target_error_label(error));
                println!("target_process_id_present=false");
                println!("target_strength=unavailable");
            }
        }

        let limit = NativeByteLimit::new(CLIPBOARD_LIMIT_BYTES).map_err(|_| "invalid_limit")?;
        match MacClipboard.read_current_text(limit) {
            Ok(text) => {
                let value = text.expose();
                println!("clipboard_read=ok");
                println!("clipboard_utf8_bytes={}", text.len_bytes());
                println!("clipboard_scalars={}", value.chars().count());
                println!(
                    "clipboard_line_controls={}",
                    value
                        .chars()
                        .filter(|value| matches!(value, '\r' | '\n'))
                        .count()
                );
                println!(
                    "clipboard_tabs={}",
                    value.chars().filter(|value| *value == '\t').count()
                );
            }
            Err(error) => {
                println!("clipboard_read={}", clipboard_error_label(error));
                println!("clipboard_utf8_bytes=unavailable");
                println!("clipboard_scalars=unavailable");
                println!("clipboard_line_controls=unavailable");
                println!("clipboard_tabs=unavailable");
            }
        }
        println!(
            "clipboard_revision_known={}",
            MacClipboard.current_revision().is_known()
        );
        println!("startup={}", startup_label(MacStartup.status()));
        println!("status=complete");
        Ok(())
    }

    fn permission_watch(seconds: u64) -> Result<(), &'static str> {
        let permission = MacAccessibility::default();
        println!("format=cliptype-p3-s01-v1");
        println!("command=permission-watch");
        println!("watch_seconds={seconds}");
        let initial = permission.state();
        println!("permission_initial={}", permission_label(initial));
        let action = permission
            .request()
            .map_err(|_| "permission_request_failed")?;
        println!("permission_action={}", permission_action_label(action));

        let deadline = Instant::now()
            .checked_add(Duration::from_secs(seconds))
            .ok_or("watch_deadline_overflow")?;
        let mut previous = initial;
        while Instant::now() < deadline {
            let current = permission.state();
            if current != previous {
                println!("permission_transition={}", permission_label(current));
                previous = current;
            }
            thread::sleep(Duration::from_millis(500));
        }
        println!("permission_final={}", permission_label(permission.state()));
        println!("status=complete");
        Ok(())
    }

    fn open_permission_settings() -> Result<(), &'static str> {
        let permission = MacAccessibility::default();
        let action = permission
            .open_system_settings()
            .map_err(|_| "open_permission_settings_failed")?;
        println!("format=cliptype-p3-s01-v1");
        println!("command=open-permission-settings");
        println!("permission_action={}", permission_action_label(action));
        println!("status=complete");
        Ok(())
    }

    fn hotkey_cycle() -> Result<(), &'static str> {
        let initial = pair("cmd+alt+shift+f17", "cmd+alt+shift+f18")?;
        let alternate = pair("cmd+alt+shift+f19", "cmd+alt+shift+f20")?;
        let (events, _receiver) = mpsc::channel::<MacMenuEvent>();
        let mut primary = MacHotkeyController::new(initial, events.clone())
            .map_err(|_| "initial_registration_failed")?;

        let occupied = MacHotkeyController::new(alternate, events.clone())
            .map_err(|_| "occupied_pair_setup_failed")?;
        let rejected = primary.replace_pair(alternate);
        let preserved_after_rejection = primary.current_pair() == initial;
        drop(occupied);

        let applied = primary.replace_pair(alternate);
        let alternate_active = primary.current_pair() == alternate;
        let restored = primary.replace_pair(initial);
        let initial_restored = primary.current_pair() == initial;

        println!("format=cliptype-p3-s01-v1");
        println!("command=hotkey-cycle");
        println!("occupied_candidate_result={}", apply_label(rejected));
        println!("old_pair_preserved_after_rejection={preserved_after_rejection}");
        println!("free_candidate_result={}", apply_label(applied));
        println!("free_candidate_active={alternate_active}");
        println!("restore_result={}", apply_label(restored));
        println!("original_pair_restored={initial_restored}");

        let passed = matches!(
            rejected,
            HotkeyApplyResult::Rejected(HotkeyAvailability::Conflict)
        ) && preserved_after_rejection
            && applied == HotkeyApplyResult::Applied
            && alternate_active
            && restored == HotkeyApplyResult::Applied
            && initial_restored;
        println!("status={}", if passed { "complete" } else { "failed" });
        if passed {
            Ok(())
        } else {
            Err("hotkey_cycle_invariant_failed")
        }
    }

    fn hold_hotkeys(trigger: &str, cancel: &str, seconds: u64) -> Result<(), &'static str> {
        let pair = pair(trigger, cancel)?;
        let (events, _receiver) = mpsc::channel::<MacMenuEvent>();
        let _controller =
            MacHotkeyController::new(pair, events).map_err(|_| "hold_registration_failed")?;
        println!("format=cliptype-p3-s01-v1");
        println!("command=hold-hotkeys");
        println!("hold_seconds={seconds}");
        println!("registration=active");
        thread::sleep(Duration::from_secs(seconds));
        println!("registration=released");
        println!("status=complete");
        Ok(())
    }

    fn status_item_smoke() -> Result<(), &'static str> {
        let (events, _receiver) = mpsc::channel::<MacMenuEvent>();
        let item = MacStatusItem::new(events).map_err(|_| "status_item_create_failed")?;
        item.update(
            true,
            InjectionMode::Auto,
            MacAccessibility::default().state(),
            false,
        );
        drop(item);
        println!("format=cliptype-p3-s01-v1");
        println!("command=status-item-smoke");
        println!("status=complete");
        Ok(())
    }

    fn startup_status() -> Result<(), &'static str> {
        println!("format=cliptype-p3-s01-v1");
        println!("command=startup-status");
        println!("startup={}", startup_label(MacStartup.status()));
        println!("status=complete");
        Ok(())
    }

    fn startup_set(enabled: bool) -> Result<(), &'static str> {
        let status = MacStartup
            .set_enabled(enabled)
            .map_err(|_| "startup_mutation_failed")?;
        println!("format=cliptype-p3-s01-v1");
        println!(
            "command={}",
            if enabled {
                "startup-enable"
            } else {
                "startup-disable"
            }
        );
        println!("startup={}", startup_label(status));
        println!("status=complete");
        Ok(())
    }

    fn pair(trigger: &str, cancel: &str) -> Result<HotkeyPair, &'static str> {
        let trigger = trigger
            .parse::<HotkeySpec>()
            .map_err(|_| "invalid_trigger")?;
        let cancel = cancel.parse::<HotkeySpec>().map_err(|_| "invalid_cancel")?;
        Ok(HotkeyPair::new(trigger, cancel))
    }

    const fn permission_label(
        value: cliptype_platform::AccessibilityPermissionState,
    ) -> &'static str {
        use cliptype_platform::AccessibilityPermissionState as State;
        match value {
            State::NotRequired => "not_required",
            State::NotRequested => "not_requested",
            State::NotGranted => "not_granted",
            State::Granted => "granted",
            State::Revoked => "revoked",
            State::Unknown => "unknown",
        }
    }

    const fn permission_action_label(
        value: cliptype_platform::PermissionActionResult,
    ) -> &'static str {
        use cliptype_platform::PermissionActionResult as Action;
        match value {
            Action::PromptRequested => "prompt_requested",
            Action::SettingsOpened => "settings_opened",
            Action::AlreadyGranted => "already_granted",
            Action::Unsupported => "unsupported",
        }
    }

    const fn capability_label(value: cliptype_core::CapabilityState) -> &'static str {
        use cliptype_core::CapabilityState as State;
        match value {
            State::Available => "available",
            State::Degraded => "degraded",
            State::Unavailable => "unavailable",
        }
    }

    const fn evidence_label(value: cliptype_core::EvidenceStrength) -> &'static str {
        use cliptype_core::EvidenceStrength as Strength;
        match value {
            Strength::TopLevelTarget => "top_level_target",
            Strength::NativeFocusedControl => "native_focused_control",
            Strength::RenderHostLimited => "render_host_limited",
            Strength::Degraded => "degraded",
        }
    }

    const fn modifier_label(value: cliptype_platform::ModifierObservation) -> &'static str {
        use cliptype_platform::ModifierObservation as Observation;
        match value {
            Observation::Clear => "clear",
            Observation::Held(_) => "held",
            Observation::Unknown => "unknown",
        }
    }

    const fn clipboard_error_label(value: cliptype_platform::ClipboardError) -> &'static str {
        use cliptype_platform::ClipboardError as Error;
        match value {
            Error::Busy => "busy",
            Error::ChangedDuringRead => "changed_during_read",
            Error::Empty => "empty",
            Error::NonText => "non_text",
            Error::Malformed => "malformed",
            Error::TooLarge { .. } => "too_large",
            Error::Native(_) => "native_error",
        }
    }

    const fn target_error_label(value: cliptype_platform::TargetCaptureError) -> &'static str {
        use cliptype_platform::TargetCaptureError as Error;
        match value {
            Error::Unavailable => "unavailable",
            Error::Disappeared => "disappeared",
            Error::Native(_) => "native_error",
        }
    }

    const fn startup_label(value: cliptype_macos::MacStartupStatus) -> &'static str {
        use cliptype_macos::MacStartupStatus as Status;
        match value {
            Status::NotRegistered => "not_registered",
            Status::Enabled => "enabled",
            Status::RequiresApproval => "requires_approval",
            Status::NotFound => "not_found",
            Status::Unsupported => "unsupported",
            Status::Unknown => "unknown",
        }
    }

    const fn apply_label(value: HotkeyApplyResult) -> &'static str {
        match value {
            HotkeyApplyResult::Applied => "applied",
            HotkeyApplyResult::Rejected(HotkeyAvailability::Available) => "rejected_available",
            HotkeyApplyResult::Rejected(HotkeyAvailability::Conflict) => "rejected_conflict",
            HotkeyApplyResult::Rejected(HotkeyAvailability::Reserved) => "rejected_reserved",
            HotkeyApplyResult::Rejected(HotkeyAvailability::Unsupported) => "rejected_unsupported",
            HotkeyApplyResult::Rejected(HotkeyAvailability::Unknown) => "rejected_unknown",
            HotkeyApplyResult::RolledBack(HotkeyAvailability::Available) => "rolled_back_available",
            HotkeyApplyResult::RolledBack(HotkeyAvailability::Conflict) => "rolled_back_conflict",
            HotkeyApplyResult::RolledBack(HotkeyAvailability::Reserved) => "rolled_back_reserved",
            HotkeyApplyResult::RolledBack(HotkeyAvailability::Unsupported) => {
                "rolled_back_unsupported"
            }
            HotkeyApplyResult::RolledBack(HotkeyAvailability::Unknown) => "rolled_back_unknown",
        }
    }
}
