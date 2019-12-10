// +--------------------------------------------------------------------------+
// | Copyright 2018 Matthew D. Steele <mdsteele@alum.mit.edu>                 |
// |                                                                          |
// | This file is part of Tachyomancer.                                       |
// |                                                                          |
// | Tachyomancer is free software: you can redistribute it and/or modify it  |
// | under the terms of the GNU General Public License as published by the    |
// | Free Software Foundation, either version 3 of the License, or (at your   |
// | option) any later version.                                               |
// |                                                                          |
// | Tachyomancer is distributed in the hope that it will be useful, but      |
// | WITHOUT ANY WARRANTY; without even the implied warranty of               |
// | MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU        |
// | General Public License for details.                                      |
// |                                                                          |
// | You should have received a copy of the GNU General Public License along  |
// | with Tachyomancer.  If not, see <http://www.gnu.org/licenses/>.          |
// +--------------------------------------------------------------------------+

mod calliope;
mod lorelei;
mod odyssey;
mod orpheus;
mod planetfall;
mod types;

use self::types::ConversationBuilder;
pub use self::types::{ConversationBubble, Portrait};
use crate::mancer::save::Profile;
use tachy::save::Conversation;

//===========================================================================//

pub trait ConversationExt {
    fn bubbles(&self, profile: &Profile) -> Vec<ConversationBubble>;
}

impl ConversationExt for Conversation {
    fn bubbles(&self, profile: &Profile) -> Vec<ConversationBubble> {
        let mut builder = ConversationBuilder::new(*self, profile);
        let _ = match *self {
            Conversation::WakeUp => odyssey::wake_up(profile, &mut builder),
            Conversation::Basics => odyssey::basics(profile, &mut builder),
            Conversation::RestorePower => {
                odyssey::restore_power(profile, &mut builder)
            }
            Conversation::MoreComponents => {
                odyssey::more_components(profile, &mut builder)
            }
            Conversation::StepTwo => odyssey::step_two(profile, &mut builder),
            Conversation::ReactorSpecs => {
                odyssey::reactor_specs(profile, &mut builder)
            }
            Conversation::WhereAreWe => {
                odyssey::where_are_we(profile, &mut builder)
            }
            Conversation::CaptainAwake => {
                odyssey::captain_awake(profile, &mut builder)
            }
            Conversation::CaptainsCall => {
                odyssey::captains_call(profile, &mut builder)
            }
            Conversation::LowVisibility => {
                odyssey::low_visibility(profile, &mut builder)
            }
            Conversation::AnIdea => odyssey::an_idea(profile, &mut builder),
            Conversation::MorePower => {
                odyssey::more_power(profile, &mut builder)
            }
            Conversation::SensorResults => {
                odyssey::sensor_results(profile, &mut builder)
            }
            Conversation::Descent => odyssey::descent(profile, &mut builder),
            Conversation::AdvancedCircuits => {
                planetfall::advanced_circuits(profile, &mut builder)
            }
            Conversation::AdditionalChips => {
                planetfall::additional_chips(profile, &mut builder)
            }
            Conversation::ScoutReport => {
                planetfall::scout_report(profile, &mut builder)
            }
            Conversation::MakingFuel => {
                planetfall::making_fuel(profile, &mut builder)
            }
            Conversation::OneMoreThing => {
                planetfall::one_more_thing(profile, &mut builder)
            }
            Conversation::WeFoundSomething => {
                planetfall::we_found_something(profile, &mut builder)
            }
            Conversation::ANewProblem => {
                planetfall::a_new_problem(profile, &mut builder)
            }
            Conversation::IncomingSignal => {
                planetfall::incoming_signal(profile, &mut builder)
            }
            Conversation::UnexpectedCompany => {
                planetfall::unexpected_company(profile, &mut builder)
            }
            Conversation::Memory => calliope::memory(profile, &mut builder),
            Conversation::KeepingTime => {
                orpheus::keeping_time(profile, &mut builder)
            }
            Conversation::CatchingUp => {
                lorelei::catching_up(profile, &mut builder)
            }
        };
        builder.build()
    }
}

//===========================================================================//
