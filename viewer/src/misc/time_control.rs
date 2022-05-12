use std::collections::{BTreeMap, BTreeSet};

use egui::NumExt;
use log_types::*;

use crate::log_db::MessageFilter;
use crate::misc::TimePoints;

use super::time_axis::TimeRange;

/// The time range we are currently zoomed in on.
#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct TimeView {
    /// Where start of the the range.
    pub min: TimeValue,

    /// How much time the full view covers.
    ///
    /// The unit is either nanoseconds or sequence numbers.
    ///
    /// If there is gaps in the data, the actual amount of viewed time might be less.
    pub time_spanned: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub(crate) enum TimeSelectionType {
    // The selection is for looping the play marker.
    Loop,
    // The selection is for viewing a bunch of data at once, replacing the play marker.
    Filter,
}

impl Default for TimeSelectionType {
    fn default() -> Self {
        Self::Loop
    }
}

impl TimeSelectionType {
    pub fn color(&self, visuals: &egui::Visuals) -> egui::Color32 {
        use egui::Color32;
        match self {
            TimeSelectionType::Loop => Color32::from_rgb(40, 200, 130),
            TimeSelectionType::Filter => visuals.selection.bg_fill, // it is a form of selection, so let's be consistent
        }
    }
}

/// State per time source.
#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
struct TimeState {
    /// The current time (play marker).
    time: TimeValue,

    /// Frames Per Second, when playing sequences (they are often video recordings).
    fps: f32,
    /// How close we are to flipping over to the next sequence number (0-1).
    sequence_t: f32,
    /// TODO: move into a `FractionalTimeValue` and use that for `Self::time`

    /// Selected time range, if any.
    #[serde(default)]
    selection: Option<TimeRange>,

    /// The time range we are currently zoomed in on.
    ///
    /// `None` means "everything", and is the default value.
    /// In this case, the view will expand while new data is added.
    /// Only when the user actually zooms or pans will this be set.
    #[serde(default)]
    view: Option<TimeView>, // TODO: use f64 for TimeValue::Sequence too, or navigation in the time panel gets weird
}

impl TimeState {
    fn new(time: TimeValue) -> Self {
        Self {
            time,
            fps: 30.0, // TODO: estimate based on data
            sequence_t: 0.0,
            selection: Default::default(),
            view: None,
        }
    }
}

/// Controls the global view and progress of the time.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(crate) struct TimeControl {
    /// Name of the time source (e.g. "log_time").
    time_source: String,

    /// For each time source.
    states: BTreeMap<String, TimeState>,

    playing: bool,
    looped: bool,
    speed: f32,

    #[serde(default)]
    pub selection_active: bool,
    #[serde(default)]
    pub selection_type: TimeSelectionType,
}

impl Default for TimeControl {
    fn default() -> Self {
        Self {
            time_source: Default::default(),
            states: Default::default(),
            playing: true,
            looped: true,
            speed: 1.0,
            selection_active: false,
            selection_type: TimeSelectionType::default(),
        }
    }
}

impl TimeControl {
    /// None when not active
    pub fn active_selection_type(&self) -> Option<TimeSelectionType> {
        self.selection_active.then(|| self.selection_type)
    }

    /// None when not active
    pub fn set_active_selection_type(&mut self, typ: Option<TimeSelectionType>) {
        match typ {
            None => {
                self.selection_active = false;
            }
            Some(typ) => {
                self.selection_active = true;
                self.selection_type = typ;
            }
        }
    }

    /// Is there a "filtering" selection, i.e. selecting a section of the time line
    pub fn is_time_filter_active(&self) -> bool {
        self.selection_active && self.selection_type == TimeSelectionType::Filter
    }

    pub fn time_source_selector_ui(&mut self, time_source_axes: &TimePoints, ui: &mut egui::Ui) {
        self.select_a_valid_time_source(time_source_axes);

        egui::ComboBox::from_id_source("time_source")
            .selected_text(&self.time_source)
            .show_ui(ui, |ui| {
                for source in time_source_axes.0.keys() {
                    if ui
                        .selectable_label(source == &self.time_source, source)
                        .clicked()
                    {
                        self.time_source = source.clone();
                    }
                }
            });

        if let Some(axis) = time_source_axes.0.get(&self.time_source) {
            if matches!(min(axis), TimeValue::Sequence(_)) {
                if let Some(state) = self.states.get_mut(&self.time_source) {
                    ui.add(
                        egui::DragValue::new(&mut state.fps)
                            .prefix("FPS: ")
                            .speed(1)
                            .clamp_range(0.0..=f32::INFINITY),
                    )
                    .on_hover_text("Frames Per Second");
                }
            }
        }
    }

    pub fn selection_ui(&mut self, ui: &mut egui::Ui) {
        use egui::SelectableLabel;

        ui.label("Selection:");

        let has_selection = self
            .states
            .get(&self.time_source)
            .map_or(false, |state| state.selection.is_some());

        if !has_selection {
            self.selection_active = false;
        }

        if ui
            .add(SelectableLabel::new(!self.selection_active, "None"))
            .on_hover_text("Disable selection")
            .clicked()
        {
            self.selection_active = false;
        }

        ui.scope(|ui| {
            ui.visuals_mut().selection.bg_fill = TimeSelectionType::Loop.color(ui.visuals());

            let is_looping =
                self.selection_active && self.selection_type == TimeSelectionType::Loop;

            if ui
                .add_enabled(has_selection, SelectableLabel::new(is_looping, "🔁"))
                .on_hover_text("Loop in selection")
                .clicked()
            {
                if is_looping {
                    self.selection_active = false; // toggle off
                } else {
                    self.set_active_selection_type(Some(TimeSelectionType::Loop));
                    self.looped = true;
                }
            }
        });

        ui.scope(|ui| {
            ui.visuals_mut().selection.bg_fill = TimeSelectionType::Filter.color(ui.visuals());

            let is_filtering =
                self.selection_active && self.selection_type == TimeSelectionType::Filter;

            if ui
                .add_enabled(has_selection, SelectableLabel::new(is_filtering, "⬌"))
                .on_hover_text("Show everything in selection")
                .clicked()
            {
                if is_filtering {
                    self.selection_active = false; // toggle off
                } else {
                    self.set_active_selection_type(Some(TimeSelectionType::Filter));
                    self.pause();
                }
            }
        });
    }

    pub fn play_pause_ui(&mut self, time_points: &TimePoints, ui: &mut egui::Ui) {
        // Toggle with space
        let anything_has_focus = ui.ctx().memory().focus().is_some();
        if !anything_has_focus
            && ui
                .input_mut()
                .consume_key(Default::default(), egui::Key::Space)
        {
            if self.playing {
                self.playing = false;
            } else {
                self.play(time_points);
            }
        }

        if ui
            .selectable_label(self.playing, "▶")
            .on_hover_text("Play. Toggle with SPACE")
            .clicked()
        {
            self.play(time_points);
        }
        if ui
            .selectable_label(!self.playing, "⏸")
            .on_hover_text("Pause. Toggle with SPACE")
            .clicked()
        {
            self.playing = false;
        }

        ui.scope(|ui| {
            ui.visuals_mut().selection.bg_fill = TimeSelectionType::Loop.color(ui.visuals());
            ui.toggle_value(&mut self.looped, "🔁")
                .on_hover_text("Loop playback");
        });

        if !self.looped && self.selection_type == TimeSelectionType::Loop {
            self.selection_active = false;
        }

        let drag_speed = (self.speed * 0.02).at_least(0.01);
        ui.add(
            egui::DragValue::new(&mut self.speed)
                .speed(drag_speed)
                .suffix("x"),
        )
        .on_hover_text("Playback speed.");

        if let Some(time_values) = time_points.0.get(self.source()) {
            let anything_has_kb_focus = ui.ctx().memory().focus().is_some();
            let step_back = ui
                .button("⏴")
                .on_hover_text("Step back to previous time with any new data (left arrow)")
                .clicked();
            let step_back = step_back
                || !anything_has_kb_focus
                    && ui
                        .input_mut()
                        .consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft);

            let step_fwd = ui
                .button("⏵")
                .on_hover_text("Step forwards to next time with any new data (right arrow)")
                .clicked();
            let step_fwd = step_fwd
                || !anything_has_kb_focus
                    && ui
                        .input_mut()
                        .consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight);

            if step_back || step_fwd {
                if let Some(time_range) = self.time_filter_range() {
                    let span = time_range.span().unwrap_or(0.0);
                    let new_min = if step_back {
                        step_back_time(&time_range.min, time_values)
                    } else {
                        step_fwd_time(&time_range.min, time_values)
                    };
                    let new_max = new_min.add_offset_f64(span);
                    self.set_time_selection(TimeRange::new(new_min, new_max));
                } else if let Some(time) = self.time() {
                    #[allow(clippy::collapsible_else_if)]
                    let new_time = if let Some(loop_range) = self.loop_range() {
                        if step_back {
                            step_back_time_looped(&time, time_values, &loop_range)
                        } else {
                            step_fwd_time_looped(&time, time_values, &loop_range)
                        }
                    } else {
                        if step_back {
                            step_back_time(&time, time_values)
                        } else {
                            step_fwd_time(&time, time_values)
                        }
                    };
                    self.set_time(new_time);
                }
            }
        }
    }

    /// Update the current time
    pub fn move_time(&mut self, egui_ctx: &egui::Context, time_points: &TimePoints) {
        self.select_a_valid_time_source(time_points);

        if !self.playing {
            return;
        }

        let full_range = if let Some(full_range) = self.full_range(time_points) {
            full_range
        } else {
            return;
        };

        let active_selection_type = self.active_selection_type();

        let state = self
            .states
            .entry(self.time_source.clone())
            .or_insert_with(|| TimeState::new(full_range.min));

        let loop_range = if self.looped && active_selection_type == Some(TimeSelectionType::Loop) {
            state.selection.unwrap_or(full_range)
        } else {
            full_range
        };

        egui_ctx.request_repaint();

        let dt = egui_ctx.input().stable_dt.at_most(0.1) * self.speed;

        if active_selection_type == Some(TimeSelectionType::Filter) {
            if let Some(time_selection) = state.selection {
                // Move filter selection

                let span = if let Some(span) = time_selection.span() {
                    span
                } else {
                    state.selection = None;
                    return;
                };

                let mut new_min = time_selection.min;

                if self.looped {
                    // max must be in the range:
                    new_min = new_min.max(loop_range.min.add_offset_f64(-span));
                }

                match &mut new_min {
                    TimeValue::Sequence(seq) => {
                        state.sequence_t += state.fps * dt;
                        *seq += state.sequence_t.floor() as i64;
                        state.sequence_t = state.sequence_t.fract();
                    }
                    TimeValue::Time(time) => {
                        *time += Duration::from_secs(dt);
                    }
                }

                if new_min > loop_range.max {
                    if self.looped {
                        // Put max just at start of loop:
                        new_min = loop_range.min.add_offset_f64(-span);
                    } else {
                        new_min = loop_range.max;
                        self.playing = false;
                    }
                }

                let new_max = new_min.add_offset_f64(span);
                state.selection = Some(TimeRange::new(new_min, new_max));

                return;
            }
        }

        // Normal time marker:

        if self.looped {
            state.time = state.time.max(loop_range.min);
        }

        match &mut state.time {
            TimeValue::Sequence(seq) => {
                state.sequence_t += state.fps * dt;
                *seq += state.sequence_t.floor() as i64;
                state.sequence_t = state.sequence_t.fract();
            }
            TimeValue::Time(time) => {
                *time += Duration::from_secs(dt);
            }
        }

        if state.time > loop_range.max {
            if self.looped {
                state.time = loop_range.min;
            } else {
                state.time = loop_range.max;
                self.playing = false;
            }
        }
    }

    fn play(&mut self, time_points: &TimePoints) {
        if self.playing {
            return;
        }

        // Start from beginning if we are at the end:
        if let Some(axis) = time_points.0.get(&self.time_source) {
            if let Some(state) = self.states.get_mut(&self.time_source) {
                if state.time >= max(axis) {
                    state.time = min(axis);
                }
            } else {
                self.states
                    .insert(self.time_source.clone(), TimeState::new(min(axis)));
            }
        }
        self.playing = true;
    }

    pub fn is_playing(&self) -> bool {
        self.playing
    }

    fn select_a_valid_time_source(&mut self, time_points: &TimePoints) {
        for source in time_points.0.keys() {
            if &self.time_source == source {
                return; // it's valid
            }
        }
        if let Some(source) = time_points.0.keys().next() {
            self.time_source = source.clone();
        } else {
            self.time_source.clear();
        }
    }

    /// The currently selected time source
    pub fn source(&self) -> &str {
        &self.time_source
    }

    /// The current time. Note that this only makes sense if there is no time selection!
    pub fn time(&self) -> Option<TimeValue> {
        if self.is_time_filter_active() {
            return None; // no single time
        }

        self.states.get(&self.time_source).map(|state| state.time)
    }

    /// The current filtered time.
    /// Returns a "point" range if we have no selection (normal play)
    pub fn time_range(&self) -> Option<TimeRange> {
        let state = self.states.get(&self.time_source)?;

        if self.is_time_filter_active() {
            state.selection
        } else {
            Some(TimeRange::point(state.time))
        }
    }

    /// If the time filter is active, what range does it cover?
    pub fn time_filter_range(&self) -> Option<TimeRange> {
        if self.is_time_filter_active() {
            self.states.get(&self.time_source)?.selection
        } else {
            None
        }
    }

    /// The current loop range, iff looping is turned on
    pub fn loop_range(&self) -> Option<TimeRange> {
        if self.selection_active && self.selection_type == TimeSelectionType::Loop {
            self.states.get(&self.time_source)?.selection
        } else {
            None
        }
    }

    /// The full range of times for the current time source
    pub fn full_range(&self, time_points: &TimePoints) -> Option<TimeRange> {
        time_points.0.get(&self.time_source).map(range)
    }

    /// Is the current time in the selection range (if any), or at the current time mark?
    pub fn is_time_selected(&self, time_source: &str, needle: TimeValue) -> bool {
        if time_source != self.time_source {
            return false;
        }

        if let Some(state) = self.states.get(&self.time_source) {
            if self.is_time_filter_active() {
                if let Some(range) = state.selection {
                    return range.contains(needle);
                }
            }

            state.time == needle
        } else {
            false
        }
    }

    pub fn set_source_and_time(&mut self, time_source: String, time: TimeValue) {
        self.time_source = time_source;
        self.set_time(time);
    }

    pub fn set_time(&mut self, time: TimeValue) {
        if self.is_time_filter_active() {
            self.selection_active = false;
        }

        self.states
            .entry(self.time_source.clone())
            .or_insert_with(|| TimeState::new(time))
            .time = time;
    }

    /// The range of time we are currently zoomed in on.
    pub fn time_view(&self) -> Option<TimeView> {
        self.states
            .get(&self.time_source)
            .and_then(|state| state.view)
    }

    /// The range of time we are currently zoomed in on.
    pub fn set_time_view(&mut self, view: TimeView) {
        self.states
            .entry(self.time_source.clone())
            .or_insert_with(|| TimeState::new(view.min))
            .view = Some(view);
    }

    /// The range of time we are currently zoomed in on.
    pub fn reset_time_view(&mut self) {
        if let Some(state) = self.states.get_mut(&self.time_source) {
            state.view = None;
        }
    }

    pub fn time_selection(&self) -> Option<TimeRange> {
        self.states.get(&self.time_source)?.selection
    }

    pub fn set_time_selection(&mut self, selection: TimeRange) {
        self.states
            .entry(self.time_source.clone())
            .or_insert_with(|| TimeState::new(selection.min))
            .selection = Some(selection);
    }

    pub fn pause(&mut self) {
        self.playing = false;
    }

    /// Return the messages that should be visible at this time.
    ///
    /// This is either based on a time selection, or it is the latest message at the current time.
    ///
    /// Returns them in arbitrary order.
    pub fn selected_messages<'db>(&self, log_db: &'db crate::log_db::LogDb) -> Vec<&'db LogMsg> {
        self.selected_messages_filtered(log_db, &MessageFilter::All)
    }

    /// Return the messages that should be visible at this time.
    ///
    /// This is either based on a time selection, or it is the latest message at the current time.
    ///
    /// Returns them in arbitrary order.
    pub fn selected_messages_for_object<'db>(
        &self,
        log_db: &'db crate::log_db::LogDb,
        object_path: &ObjectPath,
    ) -> Vec<&'db LogMsg> {
        self.selected_messages_filtered(log_db, &MessageFilter::ObjectPath(object_path.clone()))
    }

    /// Return the messages that should be visible at this time.
    ///
    /// This is either based on a time selection, or it is the latest message at the current time.
    ///
    /// Returns them in arbitrary order.
    pub fn selected_messages_filtered<'db>(
        &self,
        log_db: &'db crate::log_db::LogDb,
        filter: &MessageFilter,
    ) -> Vec<&'db LogMsg> {
        crate::profile_function!();

        let state = if let Some(state) = self.states.get(&self.time_source) {
            state
        } else {
            return Default::default();
        };

        if self.is_time_filter_active() {
            if let Some(range) = state.selection {
                return log_db.messages_in_range(self.source(), range, filter);
            }
        }

        log_db.latest_of_each_object(self.source(), state.time, filter)
    }
}

fn min(values: &BTreeSet<TimeValue>) -> TimeValue {
    *values.iter().next().unwrap()
}

fn max(values: &BTreeSet<TimeValue>) -> TimeValue {
    *values.iter().rev().next().unwrap()
}

fn range(values: &BTreeSet<TimeValue>) -> TimeRange {
    TimeRange::new(min(values), max(values))
}

fn step_fwd_time(time: &TimeValue, values: &BTreeSet<TimeValue>) -> TimeValue {
    if let Some(next) = values
        .range((std::ops::Bound::Excluded(time), std::ops::Bound::Unbounded))
        .next()
    {
        *next
    } else {
        min(values)
    }
}

fn step_fwd_time_looped(
    time: &TimeValue,
    values: &BTreeSet<TimeValue>,
    loop_range: &TimeRange,
) -> TimeValue {
    if time < &loop_range.min || &loop_range.max <= time {
        loop_range.min
    } else if let Some(next) = values
        .range((
            std::ops::Bound::Excluded(*time),
            std::ops::Bound::Included(loop_range.max),
        ))
        .next()
    {
        *next
    } else {
        step_fwd_time(time, values)
    }
}

fn step_back_time(time: &TimeValue, values: &BTreeSet<TimeValue>) -> TimeValue {
    if let Some(previous) = values.range(..time).rev().next() {
        *previous
    } else {
        max(values)
    }
}

fn step_back_time_looped(
    time: &TimeValue,
    values: &BTreeSet<TimeValue>,
    loop_range: &TimeRange,
) -> TimeValue {
    if time <= &loop_range.min || &loop_range.max < time {
        loop_range.max
    } else if let Some(previous) = values.range(loop_range.min..*time).rev().next() {
        *previous
    } else {
        step_back_time(time, values)
    }
}
