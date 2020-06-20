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

use crate::geom::CoordsSize;
use strum::IntoEnumIterator;

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PuzzleKind {
    Tutorial,
    Fabricate,
    Automate,
    Command,
    Sandbox,
}

//===========================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScoreUnits {
    Cycles,
    ManualInputs,
    Time,
    WireLength,
}

impl ScoreUnits {
    pub fn label(self) -> &'static str {
        match self {
            ScoreUnits::Cycles => "Cycles",
            ScoreUnits::ManualInputs => "Manual Inputs",
            ScoreUnits::Time => "Time",
            ScoreUnits::WireLength => "Wire Length",
        }
    }
}

//===========================================================================//

#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    EnumIter,
    EnumString,
    Eq,
    Hash,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
pub enum Puzzle {
    // Odyssey:
    TutorialOr,
    FabricateXor,
    TutorialMux,
    TutorialAdd,
    FabricateHalve,
    FabricateMul,
    AutomateHeliostat,
    AutomateGrapple,
    AutomateReactor,
    AutomateSensors,
    CommandLander,
    // Planetfall:
    TutorialDemux,
    TutorialAmp,
    TutorialSum,
    FabricateInc,
    FabricateCounter,
    AutomateMiningRobot,
    AutomateFuelSynthesis,
    AutomateBeacon,
    AutomateRobotArm,
    CommandTurret,
    // Calliope:
    TutorialRam,
    FabricateStack,
    FabricateQueue,
    AutomateCryocycler,
    AutomateInjector,
    AutomateCollector,
    AutomateTranslator,
    CommandSapper,
    // Orpheus:
    TutorialClock,
    FabricateEggTimer,
    FabricateStopwatch,
    AutomateDrillingRig,
    AutomateIncubator,
    AutomateSonar,
    AutomateResonator,
    CommandShields,
    // Lorelei:
    AutomateGeigerCounter,
    AutomateStorageDepot,
    AutomateXUnit,
    // Sandboxes:
    SandboxBehavior,
    SandboxEvent,
}

impl Puzzle {
    /// Returns the first puzzle in the game.
    pub const fn first() -> Puzzle {
        Puzzle::TutorialOr
    }

    /// Returns an iterator over all puzzles.
    pub fn all() -> PuzzleIter {
        Puzzle::iter()
    }

    pub fn title(self) -> &'static str {
        self.data().title
    }

    pub fn kind(self) -> PuzzleKind {
        self.data().kind
    }

    pub fn score_units(self) -> ScoreUnits {
        self.data().score_units
    }

    pub fn graph_bounds(self) -> (i32, u32) {
        self.data().graph_bounds
    }

    pub fn description(self) -> &'static str {
        self.data().description
    }

    pub fn instructions(self) -> &'static str {
        self.data().instructions
    }

    pub fn allows_events(self) -> bool {
        self.data().allow_events
    }

    pub fn initial_board_size(self) -> CoordsSize {
        let (width, height) = self.data().init_size;
        CoordsSize::new(width, height)
    }

    fn data(self) -> &'static PuzzleData {
        match self {
            Puzzle::AutomateBeacon => &PuzzleData {
                title: "Subspace Beacon",
                kind: PuzzleKind::Automate,
                allow_events: false,
                init_size: (8, 6),
                score_units: ScoreUnits::Time,
                graph_bounds: (200, 300),
                description:
                    "Adjust the beacon transmission angle to track the \
                     detected signal.",
                instructions: "* $!Your goal is TODO.\n\
                     * $!The optimal position is given by the sensor \
                     interface on the left side of the board.  This optimal \
                     position will change over time.\n\
                     * $!The current position is given by the motor interface \
                     on the right side of the board.\n\
                     * $!The closer the current position is to optimal, TODO.",
            },
            Puzzle::AutomateCollector => &PuzzleData {
                title: "Collector",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (8, 6),
                score_units: ScoreUnits::Time,
                graph_bounds: (500, 500),
                description: "TODO",
                instructions: "TODO",
            },
            Puzzle::AutomateCryocycler => &PuzzleData {
                title: "Cryocycler",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (9, 5),
                score_units: ScoreUnits::Time,
                graph_bounds: (500, 500),
                description: "TODO",
                instructions: "TODO",
            },
            Puzzle::AutomateDrillingRig => &PuzzleData {
                title: "Drilling Rig",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (5, 7),
                score_units: ScoreUnits::Time,
                graph_bounds: (100, 200),
                description:
                    "Regulate the speed of the drill to avoid breaking the \
                     drill head.",
                instructions: "TODO",
            },
            Puzzle::AutomateFuelSynthesis => &PuzzleData {
                title: "Fuel Synthesis",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (8, 6),
                score_units: ScoreUnits::Time,
                graph_bounds: (500, 400),
                description:
                    "Control the intake valves and mixing chamber for the \
                     hyperfuel synthesis process.",
                instructions:
                    "* $!Open each tank's intake valve until 5 units of \
                     reagent have been pumped into that tank.\n\
                     * $!Start the mixer to drain 5 units from each tank and \
                     mix them into a batch of hyperfuel.  Both intake valves \
                     must remain closed during mixing.\n\
                     * $!It is an error to allow either of the tanks to \
                     overflow.\n\
                     * $!Your goal is to produce 8 batches of fuel.",
            },
            Puzzle::AutomateGeigerCounter => &PuzzleData {
                title: "Geiger Counter",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (10, 9),
                score_units: ScoreUnits::Cycles,
                graph_bounds: (400, 5000),
                description:
                    "Calculate the number of radiation particles detected \
                     over a sliding window of time.",
                instructions: "TODO",
            },
            Puzzle::AutomateGrapple => &PuzzleData {
                title: "Grapple Launcher",
                kind: PuzzleKind::Automate,
                allow_events: false,
                init_size: (6, 8),
                score_units: ScoreUnits::Time,
                graph_bounds: (200, 200),
                description:
                    "Regulate the discharge of the grapple launcher's \
                     magnetic coils.",
                instructions: "TODO",
            },
            Puzzle::AutomateHeliostat => &PuzzleData {
                title: "Heliostat",
                kind: PuzzleKind::Automate,
                allow_events: false,
                init_size: (6, 5),
                score_units: ScoreUnits::Time,
                graph_bounds: (100, 200),
                description:
                    "Position the ship's heliostat to reflect sunlight \
                     onto the solar panels at the optimal angle.",
                instructions:
                    "* $!Your goal is to fill the energy meter by always \
                     moving the heliostat towards the optimal position.\n\
                     * $!The optimal position is given by the sensor \
                     interface on the left side of the board.  This optimal \
                     position will change over time.\n\
                     * $!The current position is given by the motor \
                     interface on the right side of the board.\n\
                     * $!The closer the current position is to optimal, the \
                     more energy will be produced.",
            },
            Puzzle::AutomateIncubator => &PuzzleData {
                title: "Incubator",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (9, 6),
                score_units: ScoreUnits::Time,
                graph_bounds: (500, 500),
                description:
                    "Incubate the eggs of flora native to this planet in \
                     order to render them edible for Ichthyans.",
                instructions:
                    "* $!The incubator holds up to two eggs at once.  You \
                     must incubate 10 eggs (5 on each side).\n\
                     * $!Each side will signal when a new egg is ready to be \
                     loaded on that side.\n\
                     * $!Turn on the heater to warm any eggs inside.  Each \
                     egg must be warmed for exactly 20 time steps (not \
                     necessarily contiguous), then unloaded.\n\
                     * $!Loading or unloading an egg takes a few time \
                     steps.  The heater must be off during this time.",
            },
            Puzzle::AutomateInjector => &PuzzleData {
                title: "Plasma Injector",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (9, 5),
                score_units: ScoreUnits::Time,
                graph_bounds: (500, 500),
                description:
                    "Control the main reactor's primary plasma injection head \
                     in order to maintain a steady Low-Temperature Fusion \
                     (LTF) reaction.",
                instructions: "TODO",
            },
            Puzzle::AutomateMiningRobot => &PuzzleData {
                title: "Mining Robot",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (8, 6),
                score_units: ScoreUnits::Time,
                graph_bounds: (100, 500),
                description:
                    "Program the scout robot to pick up ore deposits and \
                     carry them back to the base.",
                instructions:
                    "* $!Your goal is carry all the ore back to the base.\n\
                     * $!The robot will depart from the base in a straight \
                     line, and can dig up ore deposits while passing \
                     over them.  It will return to the base on command, \
                     or automatically if it goes out too far.\n\
                     * $!The robot can hold up to 15kg of ore at once.  It \
                     is an error to try to carry more than that.\n\
                     * $!When the robot returns to the base, it will \
                     automatically dump its ore and depart again.",
            },
            Puzzle::AutomateReactor => &PuzzleData {
                title: "Backup Reactor",
                kind: PuzzleKind::Automate,
                allow_events: false,
                init_size: (6, 8),
                score_units: ScoreUnits::Time,
                graph_bounds: (400, 500),
                description:
                    "Manipulate the reactor's control rods to adjust the \
                     power output to the desired level.",
                instructions: "TODO",
            },
            Puzzle::AutomateResonator => &PuzzleData {
                title: "Strike Resonator",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (9, 7),
                score_units: ScoreUnits::Time,
                graph_bounds: (500, 500),
                description:
                    "Deliver timed pulses to a resonator crystal in order to \
                     charge up the STRIKE beam weapon.",
                instructions: "TODO",
            },
            Puzzle::AutomateRobotArm => &PuzzleData {
                title: "Manipulator Arm",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (8, 6),
                score_units: ScoreUnits::Time,
                graph_bounds: (300, 500),
                description:
                    "Operate a robotic arm in response to radio commands.",
                instructions: "TODO",
            },
            Puzzle::AutomateSensors => &PuzzleData {
                title: "Main Sensors",
                kind: PuzzleKind::Automate,
                allow_events: false,
                init_size: (6, 7),
                score_units: ScoreUnits::Time,
                graph_bounds: (150, 150),
                description:
                    "Narrow the sensor sweep to zero in on a given signal of \
                     interest.",
                instructions:
                    "* $!The two inputs indicate the current upper and \
                     lower bounds (inclusive) of the scan range.\n\
                     * $!Your goal is to subdivide the scan range at each \
                     step, until the final value is found.  The final \
                     value will always be odd.\n\
                     * $!When the bounds are 2 or more apart, output \
                     any value strictly between the two.\n\
                     * $!When the bounds are exactly 1 apart, output \
                     whichever of those two values is odd.\n\
                     * $!When the bounds are equal, output that final \
                     value to terminate the scan.\n\
                     * $!Note that ($/x$/ AND 1) is 0 when $/x$/ is even, \
                     and 1 when $/x$/ is odd.",
            },
            Puzzle::AutomateSonar => &PuzzleData {
                title: "Sonar Navigation",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (9, 7),
                score_units: ScoreUnits::Time,
                graph_bounds: (500, 1000),
                description:
                    "Use sonar to guide an autonomous underwater vehicle \
                     through a submarine canyon.",
                instructions: "TODO",
            },
            Puzzle::AutomateStorageDepot => &PuzzleData {
                title: "Storage Depot",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (8, 6),
                score_units: ScoreUnits::Time,
                graph_bounds: (500, 1000),
                description:
                    "Store crates within a warehouse and retrieve them again \
                     on demand.",
                instructions: "TODO",
            },
            Puzzle::AutomateTranslator => &PuzzleData {
                title: "Translator",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (8, 5),
                score_units: ScoreUnits::Time,
                graph_bounds: (500, 500),
                description: "Translate a passage of Ichthyan text using a \
                              word-for-word dictionary database.",
                instructions: "TODO",
            },
            Puzzle::AutomateXUnit => &PuzzleData {
                title: "X-Unit",
                kind: PuzzleKind::Automate,
                allow_events: true,
                init_size: (12, 9),
                score_units: ScoreUnits::Time,
                graph_bounds: (300, 300),
                description:
                    "Ensure that all the implosion charges surrounding the \
                     fissile core detonate simultaneously.",
                instructions:
                    "* $!Your goal is to make all 256 charges detonate at \
                     once.\n\
                     * $!Each charge has some delay between when the ignition \
                     signal is sent and when it detonates.\n\
                     * $!You must stagger the Fire events so that all charges \
                     detonate in the same time step.\n\
                     * $!You can measure the delay for a charge by sending a \
                     test signal to the Ping port and waiting for an echo on \
                     the Pong port.  The echo will take exactly twice that \
                     charge's delay to come back.",
            },
            Puzzle::CommandLander => &PuzzleData {
                title: "Orbital Lander",
                kind: PuzzleKind::Command,
                allow_events: false,
                init_size: (6, 8),
                score_units: ScoreUnits::ManualInputs,
                graph_bounds: (200, 100),
                description:
                    "Operate the lander thrusters during descent for a soft \
                     landing.",
                instructions: "TODO",
            },
            Puzzle::CommandSapper => &PuzzleData {
                title: "Sapper Drone",
                kind: PuzzleKind::Command,
                allow_events: true,
                init_size: (9, 7),
                score_units: ScoreUnits::ManualInputs,
                graph_bounds: (500, 500),
                description:
                    "Pilot a remote drone through the minefield to each of \
                     the control satellites to enable the mines' cloaking \
                     devices.",
                instructions: "TODO",
            },
            Puzzle::CommandShields => &PuzzleData {
                title: "Deflector Shields",
                kind: PuzzleKind::Command,
                allow_events: true,
                init_size: (9, 7),
                score_units: ScoreUnits::ManualInputs,
                graph_bounds: (500, 100),
                description:
                    "Control the ship's shields to block incoming enemy \
                     torpedoes, then return fire.",
                instructions: "TODO",
            },
            Puzzle::CommandTurret => &PuzzleData {
                title: "Defense Turret",
                kind: PuzzleKind::Command,
                allow_events: true,
                init_size: (9, 7),
                score_units: ScoreUnits::ManualInputs,
                graph_bounds: (500, 100),
                description:
                    "Aim and fire the pulse cannon turret to fend off waves \
                     of enemy attackers.",
                instructions: "TODO",
            },
            Puzzle::FabricateCounter => &PuzzleData {
                title: "Counter",
                kind: PuzzleKind::Fabricate,
                allow_events: true,
                init_size: (7, 5),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (150, 150),
                description:
                    "Build a memory chip that can increment or decrement its \
                     value in response to events.\n\n\
                     Once this task is completed, you will be able to use \
                     $*Counter$* chips in future tasks.",
                instructions: "TODO",
            },
            Puzzle::FabricateEggTimer => &PuzzleData {
                title: "Egg Timer",
                kind: PuzzleKind::Fabricate,
                allow_events: true,
                init_size: (7, 7),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (150, 200),
                description:
                    "Build a timing chip that counts down from a given time \
                     and then fires an event at zero.\n\n\
                     Once this task is completed, you will be able to use \
                     $*Egg Timer$* chips in future tasks.",
                instructions: "TODO",
            },
            Puzzle::FabricateHalve => &PuzzleData {
                title: "Halver",
                kind: PuzzleKind::Fabricate,
                allow_events: false,
                init_size: (7, 5),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (100, 100),
                description:
                    "Build a 4-bit halver using packers and unpackers.\n\n\
                     Once this task is completed, you will be able to use \
                     generic $*Halve$* chips in future tasks.",
                instructions:
                    "* $!The output should be half the value of the input, \
                     rounded down.\n\
                     * $!This can be achieved by unpacking the input into \
                     four 1-bit wires, then packing the highest three wires \
                     of the input into the lowest three wires of the output.",
            },
            Puzzle::FabricateInc => &PuzzleData {
                title: "Incrementor",
                kind: PuzzleKind::Fabricate,
                allow_events: true,
                init_size: (5, 5),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (100, 100),
                description:
                    "Build a 4-bit incrementor using more basic event and \
                     behavior chips.\n\n\
                     Once this task is completed, you will be able to use \
                     generic $*Inc$* chips in future tasks.",
                instructions:
                    "* $!Your goal is to construct an incrementor.\n\
                     * $!When an input event arrives, the circuit should \
                     add the input behavior value to the event value, \
                     and emit an output event with the sum.",
            },
            Puzzle::FabricateMul => &PuzzleData {
                title: "Multiplier",
                kind: PuzzleKind::Fabricate,
                allow_events: false,
                init_size: (7, 7),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (150, 200),
                description:
                    "Build an 8-bit multiplier using 4-bit multipliers.\n\n\
                     Once this task is completed, you will be able to use \
                     generic $*Mul$* chips in future tasks.",
                instructions:
                    "* $!Your goal is to construct an 8-bit multiplier.\n\
                     * $!The output should be the product of $*In1$* and \
                     $*In2$*.  This product will never be more than 255.",
            },
            Puzzle::FabricateQueue => &PuzzleData {
                title: "Queue Memory",
                kind: PuzzleKind::Fabricate,
                allow_events: true,
                init_size: (7, 6),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (500, 500),
                description:
                    "Construct a simple first-in, first-out queue memory \
                     module.\n\n\
                     Once this task is completed, you will be able to use \
                     $*Queue$* chips in future tasks.",
                instructions: "TODO",
            },
            Puzzle::FabricateStack => &PuzzleData {
                title: "Stack Memory",
                kind: PuzzleKind::Fabricate,
                allow_events: true,
                init_size: (7, 6),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (500, 500),
                description:
                    "Construct a simple last-in, first-out stack memory \
                     module.\n\n\
                     Once this task is completed, you will be able to use \
                     $*Stack$* chips in future tasks.",
                instructions: "TODO",
            },
            Puzzle::FabricateStopwatch => &PuzzleData {
                title: "Stopwatch",
                kind: PuzzleKind::Fabricate,
                allow_events: true,
                init_size: (7, 7),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (150, 200),
                description:
                    "Build a timing chip that counts upwards from zero, and \
                     that can be paused or reset.\n\n\
                     Once this task is completed, you will be able to use \
                     $*Stopwatch$* chips in future tasks.",
                instructions: "TODO",
            },
            Puzzle::FabricateXor => &PuzzleData {
                title: "XOR Gate",
                kind: PuzzleKind::Fabricate,
                allow_events: false,
                init_size: (5, 5),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (100, 100),
                description: "Build a 1-bit $*XOR$* gate out of $*AND$*, \
                     $*OR$*, and $*NOT$* gates.\n\n\
                     Once this task is completed, you will be able to use \
                     $*XOR$* gates in future tasks.",
                instructions:
                    "* $!Your goal is to construct a $*XOR$* gate.\n\
                     * $!The output on the right side of the board should \
                     be 1 if exactly one input is 1, but not both.\n\
                     * $!Note that ($/a$/ XOR $/b$/) is equivalent to \
                     ($/a$/ OR $/b$/) AND NOT ($/a$/ AND $/b$/).",
            },
            Puzzle::SandboxBehavior => &PuzzleData {
                title: "Behavior Lab",
                kind: PuzzleKind::Sandbox,
                allow_events: false,
                init_size: (8, 6),
                score_units: ScoreUnits::Time,
                graph_bounds: (100, 100),
                description:
                    "Build any circuits you want using all behavior chips \
                     that are currently available.  You can use this area \
                     for prototyping, experimentation, or freeform design.",
                instructions: "",
            },
            Puzzle::SandboxEvent => &PuzzleData {
                title: "Event Lab",
                kind: PuzzleKind::Sandbox,
                allow_events: true,
                init_size: (8, 6),
                score_units: ScoreUnits::Time,
                graph_bounds: (100, 100),
                description:
                    "Build any circuits you want using all behavior and \
                     event chips that are currently available.  You can \
                     use this area for prototyping, experimentation, or \
                     freeform design.",
                instructions: "",
            },
            Puzzle::TutorialAdd => &PuzzleData {
                title: "Adder",
                kind: PuzzleKind::Tutorial,
                allow_events: false,
                init_size: (5, 5),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (100, 100),
                description:
                    "Build a 4-bit adder using 2-bit adders, packers, and \
                     unpackers.\n\n\
                     Once this task is completed, you will be able to use \
                     generic $*Add$* chips in future tasks.",
                instructions: "* $!Your goal is to construct a 4-bit adder.\n\
                     * $!The output should be the sum of $*In1$* and \
                     $*In2$*.  This sum will never be more than 15.\n\
                     * $!You can use $*Unpack$* chips to separate the \
                     4-bit inputs into hi and lo 2-bit values that the \
                     2-bit adders can accept.  Remember to handle carry \
                     bits appropriately.",
            },
            Puzzle::TutorialAmp => &PuzzleData {
                title: "Signal Amplifier",
                kind: PuzzleKind::Tutorial,
                allow_events: true,
                init_size: (6, 5),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (150, 150),
                description:
                    "Construct a signal amplifier with an automatic safety \
                     cutoff for high-intensity signals.",
                instructions:
                    "* $!When an event arrives on the left side of the board, \
                     double its value and send it out the right side of the \
                     board.\n\
                     * $!However, if the doubled value is greater than 10, no \
                     event should be sent.",
            },
            Puzzle::TutorialClock => &PuzzleData {
                title: "Noise Filter",
                kind: PuzzleKind::Tutorial,
                allow_events: true,
                init_size: (6, 5),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (150, 200),
                description:
                    "Construct a simple signal processor that filters out \
                     overly-noisy inputs.",
                instructions:
                    "* $!When an event arrives on the left side of the board, \
                     send it out the right side of the board after a delay of \
                     two time steps.\n\
                     * $!If another event arrives in the middle of this delay \
                     period, it should be ignored.",
            },
            Puzzle::TutorialDemux => &PuzzleData {
                title: "Four-Way Demux",
                kind: PuzzleKind::Tutorial,
                allow_events: true,
                init_size: (6, 4),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (150, 200),
                description:
                    "Route incoming events to one of four destinations, based \
                     on a 2-bit control value.",
                instructions: "TODO",
            },
            Puzzle::TutorialMux => &PuzzleData {
                title: "Two-Way Mux",
                kind: PuzzleKind::Tutorial,
                allow_events: false,
                init_size: (5, 5),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (100, 100),
                description:
                    "Build a 1-bit multiplexer using other logic gates.\n\n\
                     Once this task is completed, you will be able to use \
                     $*Mux$* chips in future tasks.",
                instructions: "* $!Your goal is to construct a 1-bit Mux.\n\
                     * $!The output should be the value of $*in0$* if \
                     $*ctrl$* is 0, or of $*in1$* if $*ctrl$* is 1.\n\
                     * $!If $/a$/ and $/b$/ are the inputs and $/c$/ is \
                     the control, then a Mux is    \
                     ($/a$/ AND NOT $/c$/) OR ($/b$/ AND $/c$/).",
            },
            Puzzle::TutorialOr => &PuzzleData {
                title: "OR Gate",
                kind: PuzzleKind::Tutorial,
                allow_events: false,
                init_size: (5, 5),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (50, 50),
                description:
                    "Build a 1-bit $*OR$* gate out of $*AND$* and $*NOT$* \
                     gates.\n\n\
                     Once this task is completed, you will be able to use \
                     $*OR$* gates in future tasks.",
                instructions:
                    "* $!Your goal is to construct an $*OR$* gate.\n\
                     * $!The output on the right side of the board should \
                     be 1 if either input is 1, or 0 if both inputs are 0.\n\
                     * $!Note that ($/a$/ OR $/b$/) is equivalent to \
                     NOT ((NOT $/a$/) AND (NOT $/b$/)).",
            },
            Puzzle::TutorialRam => &PuzzleData {
                title: "Repeat Filter",
                kind: PuzzleKind::Tutorial,
                allow_events: true,
                init_size: (6, 5),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (200, 200),
                description:
                    "Construct a simple signal processor that filters out \
                     repeated inputs.",
                instructions: "TODO",
            },
            Puzzle::TutorialSum => &PuzzleData {
                title: "Running Total",
                kind: PuzzleKind::Tutorial,
                allow_events: true,
                init_size: (7, 7),
                score_units: ScoreUnits::WireLength,
                graph_bounds: (150, 200),
                description:
                    "Build a circuit for tracking a running total and \
                     resetting it back to zero.",
                instructions:
                    "* $!The output should start at zero.  When an input \
                     event arrives, its value should be added to the \
                     running total output.\n\
                     * $!When a reset event arrives, the output should be \
                     reset back to zero.\n\
                     * $!If a reset and input event arrive simultaneously, \
                     the reset should occur just $/before$/ the input is \
                     added in, so that the output for that time step is \
                     the value of the input event.",
            },
        }
    }
}

//===========================================================================//

struct PuzzleData {
    title: &'static str,
    kind: PuzzleKind,
    allow_events: bool,
    init_size: (i32, i32),
    score_units: ScoreUnits,
    graph_bounds: (i32, u32),
    description: &'static str,
    instructions: &'static str,
}

//===========================================================================//

#[cfg(test)]
mod tests {
    use super::Puzzle;
    use std::str::FromStr;

    #[test]
    fn puzzle_to_and_from_string() {
        for puzzle in Puzzle::all() {
            let string = format!("{:?}", puzzle);
            assert_eq!(Puzzle::from_str(&string), Ok(puzzle));
        }
    }

    #[test]
    fn puzzle_kinds() {
        for puzzle in Puzzle::all() {
            let name = format!("{:?}", puzzle);
            let kind = format!("{:?}", puzzle.kind());
            assert!(name.starts_with(kind.as_str()));
        }
    }
}

//===========================================================================//
