use bevy::prelude::*;
use saddle_animation_text_animation::TextAnimationAction;
use saddle_bevy_e2e::{action::Action, actions::assertions, scenario::Scenario};

use crate::{LabDiagnostics, send_command_to_named, set_reduced_motion};

pub fn list_scenarios() -> Vec<&'static str> {
    vec![
        "smoke_launch",
        "typewriter_showcase",
        "layered_effects_showcase",
        "reduced_motion_showcase",
        "unicode_showcase",
        "stress_showcase",
    ]
}

pub fn scenario_by_name(name: &str) -> Option<Scenario> {
    match name {
        "smoke_launch" => Some(smoke_launch()),
        "typewriter_showcase" => Some(typewriter_showcase()),
        "layered_effects_showcase" => Some(layered_effects_showcase()),
        "reduced_motion_showcase" => Some(reduced_motion_showcase()),
        "unicode_showcase" => Some(unicode_showcase()),
        "stress_showcase" => Some(stress_showcase()),
        _ => None,
    }
}

fn command(name: &'static str, action: TextAnimationAction) -> Action {
    Action::Custom(Box::new(move |world| {
        send_command_to_named(world, name, action);
    }))
}

fn reduced_motion(enabled: bool) -> Action {
    Action::Custom(Box::new(move |world| set_reduced_motion(world, enabled)))
}

fn smoke_launch() -> Scenario {
    Scenario::builder("smoke_launch")
        .description("Boot the crate-local lab, verify the core showcase blocks initialize, and capture the baseline scene.")
        .then(Action::WaitFrames(10))
        .then(assertions::resource_exists::<LabDiagnostics>("lab diagnostics exist"))
        .then(assertions::custom("core showcase elements initialized", |world| {
            let diagnostics = world.resource::<LabDiagnostics>();
            diagnostics.dialogue_total_units > 0
                && diagnostics.unicode_total_units > 0
                && diagnostics.headline_effect_count >= 2
                && diagnostics.stress_label_count >= 80
        }))
        .then(assertions::custom("at least one reveal checkpoint has fired", |world| {
            world.resource::<LabDiagnostics>().checkpoint_count > 0
        }))
        .then(Action::Screenshot("smoke".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("smoke_launch"))
        .build()
}

fn typewriter_showcase() -> Scenario {
    Scenario::builder("typewriter_showcase")
        .description("Restart the dialogue reveal, capture the start, a mid-reveal checkpoint, then finish instantly and verify completion.")
        .then(command("Dialogue Typewriter", TextAnimationAction::Restart))
        .then(Action::WaitFrames(1))
        .then(Action::Screenshot("typewriter_start".into()))
        .then(Action::WaitFrames(60))
        .then(assertions::custom("dialogue reaches a mid-reveal checkpoint with runtime markup effects and sound hooks", |world| {
            let diagnostics = world.resource::<LabDiagnostics>();
            diagnostics.dialogue_visible_units > 0
                && diagnostics.dialogue_visible_units < diagnostics.dialogue_total_units
                && diagnostics.dialogue_effect_count >= 3
                && diagnostics.sound_request_count > 0
                && diagnostics.last_sound_cue.as_deref() == Some("lab.dialogue.blip")
        }))
        .then(Action::Screenshot("typewriter_mid".into()))
        .then(Action::WaitFrames(1))
        .then(command("Dialogue Typewriter", TextAnimationAction::FinishNow))
        .then(Action::WaitFrames(4))
        .then(assertions::custom("dialogue reveal completes", |world| {
            let diagnostics = world.resource::<LabDiagnostics>();
            diagnostics.last_completed_name.as_deref() == Some("Dialogue Typewriter")
                && diagnostics.dialogue_visible_units == diagnostics.dialogue_total_units
        }))
        .then(Action::Screenshot("typewriter_done".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("typewriter_showcase"))
        .build()
}

fn layered_effects_showcase() -> Scenario {
    Scenario::builder("layered_effects_showcase")
        .description("Capture the decorative headline and layered world label at two different moments while confirming the configured effect stack is active.")
        .then(Action::WaitFrames(12))
        .then(assertions::custom("headline effect stack is configured", |world| {
            world.resource::<LabDiagnostics>().headline_effect_count >= 2
        }))
        .then(Action::Screenshot("layered_a".into()))
        .then(Action::WaitFrames(36))
        .then(assertions::custom("world label remains visible during decorative motion", |world| {
            world.resource::<LabDiagnostics>().world_visible_graphemes > 0
        }))
        .then(Action::Screenshot("layered_b".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("layered_effects_showcase"))
        .build()
}

fn reduced_motion_showcase() -> Scenario {
    Scenario::builder("reduced_motion_showcase")
        .description("Capture the full-motion state, enable reduced motion globally, and capture the reduced-motion variant.")
        .then(reduced_motion(false))
        .then(Action::WaitFrames(8))
        .then(assertions::custom("reduced motion starts disabled", |world| {
            !world.resource::<LabDiagnostics>().reduced_motion
        }))
        .then(Action::Screenshot("reduced_motion_off".into()))
        .then(Action::WaitFrames(1))
        .then(reduced_motion(true))
        .then(Action::WaitFrames(8))
        .then(assertions::custom("reduced motion is enabled", |world| {
            world.resource::<LabDiagnostics>().reduced_motion
        }))
        .then(Action::Screenshot("reduced_motion_on".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("reduced_motion_showcase"))
        .build()
}

fn unicode_showcase() -> Scenario {
    Scenario::builder("unicode_showcase")
        .description("Restart the Unicode sample, capture start and mid-reveal, then finish it and verify the full multilingual string is revealed.")
        .then(command("Unicode Sample", TextAnimationAction::Restart))
        .then(Action::WaitFrames(1))
        .then(Action::Screenshot("unicode_start".into()))
        .then(Action::WaitFrames(20))
        .then(assertions::custom("unicode sample reveals progressively", |world| {
            let diagnostics = world.resource::<LabDiagnostics>();
            diagnostics.unicode_visible_units > 0
                && diagnostics.unicode_visible_units < diagnostics.unicode_total_units
        }))
        .then(Action::Screenshot("unicode_mid".into()))
        .then(Action::WaitFrames(1))
        .then(command("Unicode Sample", TextAnimationAction::FinishNow))
        .then(Action::WaitFrames(4))
        .then(assertions::custom("unicode sample fully reveals", |world| {
            let diagnostics = world.resource::<LabDiagnostics>();
            diagnostics.unicode_visible_units == diagnostics.unicode_total_units
        }))
        .then(Action::Screenshot("unicode_done".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("unicode_showcase"))
        .build()
}

fn stress_showcase() -> Scenario {
    Scenario::builder("stress_showcase")
        .description("Verify the stress field initializes a high label count and capture the settled scene at two different moments.")
        .then(Action::WaitFrames(12))
        .then(assertions::custom("stress field spawned all labels", |world| {
            world.resource::<LabDiagnostics>().stress_label_count >= 80
        }))
        .then(Action::Screenshot("stress_a".into()))
        .then(Action::WaitFrames(30))
        .then(assertions::custom("stress field stays populated", |world| {
            world.resource::<LabDiagnostics>().stress_label_count >= 80
        }))
        .then(Action::Screenshot("stress_b".into()))
        .then(Action::WaitFrames(1))
        .then(assertions::log_summary("stress_showcase"))
        .build()
}
