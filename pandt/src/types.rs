//! Simple types, with pure operations.

use std::collections::{HashMap, VecDeque, HashSet};
use string_wrapper::StringWrapper;

use rand;
use rand::distributions as dist;
use rand::distributions::IndependentSample;

use uuid::{Uuid, ParseError as UuidParseError};

use serde::ser;
use serde::ser::{SerializeStruct, Error as SerError};

use nonempty;
use indexed::{DeriveKey, IndexedHashMap};
use foldertree::{FolderTree, FolderPath, FolderTreeError, FolderTreeErrorKind};

/// Point3 defines a 3d position in meters.
pub type Point3 = (i16, i16, i16);
pub type ConditionID = usize;
pub type Color = String;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Serialize, Deserialize)]
pub struct Dice {
  pub num: u8,
  pub size: u8,
}

impl Dice {
  pub fn flat(val: u8) -> Dice {
    Dice {
      num: val,
      size: 1,
    }
  }
  pub fn roll(&self) -> (Vec<u8>, u32) {
    let mut intermediate = vec![];
    let mut result = 0u32;
    let range: dist::Range<u8> = dist::Range::new(1, self.size + 1);
    let mut rng = rand::thread_rng();
    for _ in 0..self.num {
      let val = range.ind_sample(&mut rng);
      result += val as u32;
      intermediate.push(val);
    }
    (intermediate, result)
  }
}

#[derive(Add, Sub, Mul, Div, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Serialize,
         Deserialize)]
pub struct HP(pub u8);
impl HP {
  pub fn saturating_add(self, other: Self) -> Self {
    HP(self.0.saturating_add(other.0))
  }
  pub fn saturating_sub(self, other: Self) -> Self {
    HP(self.0.saturating_sub(other.0))
  }
}

#[derive(Add, Sub, Mul, Div, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug, Hash, Serialize,
         Deserialize)]
pub struct Energy(pub u8);
impl Energy {
  pub fn saturating_add(self, other: Self) -> Self {
    Energy(self.0.saturating_add(other.0))
  }
  pub fn saturating_sub(self, other: Self) -> Self {
    Energy(self.0.saturating_sub(other.0))
  }
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct PlayerID(pub String);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct CreatureID(Uuid);
impl CreatureID {
  pub fn new() -> CreatureID {
    CreatureID(Uuid::new_v4())
  }
  pub fn from_str(s: &str) -> Result<CreatureID, GameError> {
    Ok(CreatureID(Uuid::parse_str(s).map_err(|_| GameErrorEnum::CreatureNotFound(s.to_string()))?))
  }
  pub fn to_string(&self) -> String {
    self.0.hyphenated().to_string()
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct SceneID(Uuid);
impl SceneID {
  pub fn new() -> SceneID {
    SceneID(Uuid::new_v4())
  }
  pub fn from_str(s: &str) -> Result<SceneID, GameError> {
    Ok(SceneID(Uuid::parse_str(s)?))
  }
  pub fn to_string(&self) -> String {
    self.0.hyphenated().to_string()
  }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct MapID(Uuid);
impl MapID {
  pub fn new() -> MapID {
    MapID(Uuid::new_v4())
  }
  pub fn from_str(s: &str) -> Result<MapID, GameError> {
    Ok(MapID(Uuid::parse_str(s)?))
  }
  pub fn to_string(&self) -> String {
    self.0.hyphenated().to_string()
  }
}


#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub struct AbilityID(StringWrapper<[u8; 64]>);
impl AbilityID {
  pub fn new(s: &str) -> Result<Self, GameError> {
    let sw =
      StringWrapper::from_str_safe(s).ok_or_else(|| GameErrorEnum::IDTooLong(s[..64].to_string()))?;
    Ok(AbilityID(sw))
  }
  pub fn to_string(&self) -> String {
    self.0.to_string()
  }
}

#[cfg(test)]
pub fn abid(s: &str) -> AbilityID {
  AbilityID::new(s).unwrap()
}


/// Distance in centimeters.
#[derive(Add, Sub, Mul, Div, Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash,
         Serialize, Deserialize)]
pub struct Distance(pub u32);
impl Distance {
  /// Convert meters as a f32 to a Distance.
  pub fn from_meters(x: f32) -> Distance {
    Distance((x * 100.0) as u32)
  }
  pub fn saturating_add(self, other: Self) -> Self {
    Distance(self.0.saturating_add(other.0))
  }
  pub fn saturating_sub(self, other: Self) -> Self {
    Distance(self.0.saturating_sub(other.0))
  }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum FolderItemID {
  SceneID(SceneID),
  MapID(MapID),
  CreatureID(CreatureID),
  NoteID(String),
  SubfolderID(String),
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum SkillLevel {
  // The way to read these are:
  // "A {variant} person has a 75% chance of doing this."
  // If you're unskilled and you are doing a Skilled difficulty challenge: 50%?
  // Trivial to Expert: 10%?
  Inept,
  Unskilled,
  Skilled,
  Expert,
  Supernatural,
}

impl SkillLevel {
  pub fn to_ord(&self) -> i8 {
    match self {
      Inept => -1,
      Unskilled => 0,
      Skilled => 1,
      Expert => 2,
      Supernatural => 3,
    }
  }

  pub fn challenge(&self, difficulty_level: SkillLevel) -> u8 {
    match difficulty_level.to_ord() - self.to_ord() {
      -4 => 100,
      -3 => 99,
      -2 => 95,
      -1 => 85,
      0 => 75,
      1 => 50,
      2 => 10,
      3 => 1,
      4 => 0,
      diff => panic!("[SkillLevel::challenge] Two skill levels were too far apart: {:?}", diff),
    }
  }
}


/// Top-level commands that can be sent from a client to affect the state of the app.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameCommand {
  SimpleAttributeCheck(CreatureID, AttrID, SkillLevel),
  RandomAttributeCheck(CreatureID, AttrID, SkillLevel),

  /// Create a folder, given segments leading to it.
  CreateFolder(FolderPath),
  /// Rename a folder.
  RenameFolder(FolderPath, String),
  /// Delete a folder.
  DeleteFolder(FolderPath),

  /// Move some object from one folder to another.
  MoveFolderItem(FolderPath, FolderItemID, FolderPath),

  /// Create a Note inside of a Folder.
  CreateNote(FolderPath, Note),
  /// Rename a Note inside of a Folder.
  EditNote(FolderPath, String, Note),
  /// Delete a Note from a Folder.
  DeleteNote(FolderPath, String),

  // ** Scene management **
  /// Create a Scene.
  CreateScene(FolderPath, SceneCreation),
  /// Edit a scene. The ID in the given Scene must match an existing Scene.
  EditScene(Scene),
  /// Delete a scene.
  DeleteScene(SceneID),

  // ** Map management **
  CreateMap(FolderPath, MapCreation),
  /// Change a map. The ID of the given map bust match an existing map.
  EditMap(Map),
  /// Delete a map.
  DeleteMap(MapID),

  // ** Combat management **
  /// Start a combat with the specified creatures.
  StartCombat(SceneID, Vec<CreatureID>),
  /// Stop the current combat.
  StopCombat,
  /// Add a creature to combat. Combat must already be running; otherwise use `StartCombat`.
  AddCreatureToCombat(CreatureID),
  /// Remove a creature from combat. Combat must already be running.
  RemoveCreatureFromCombat(CreatureID),
  /// Use an Ability out of combat.
  ActCreature(SceneID, CreatureID, AbilityID, DecidedTarget),
  /// Make the current creature use an ability.
  CombatAct(AbilityID, DecidedTarget),
  /// Move the current creature in combat to a point.
  /// There must be a clear path according to the current loaded map.
  PathCurrentCombatCreature(Point3),
  /// End the current creature's turn.
  Done,
  /// Modify a creature's order in the combat list.
  ChangeCreatureInitiative(CreatureID, usize),

  // ** Creature Manipulation **
  /// Create a new creature.
  CreateCreature(FolderPath, CreatureCreation),
  /// Edit an existing creature.
  EditCreature(Creature),
  /// Assign a creature's position within a scene.
  SetCreaturePos(SceneID, CreatureID, Point3),
  /// Move a creature along a path within a scene.
  /// There must be a clear path according to the current loaded map. It doesn't matter whether
  /// the creature is in combat.
  PathCreature(SceneID, CreatureID, Point3),
  /// Remove a creature from the game entirely. Creature must not be in combat.
  DeleteCreature(CreatureID),

  // ** Player Manipulation **
  /// Register a player as available for controlling a creature.
  RegisterPlayer(PlayerID),
  /// Give control of a creature to a player.
  GiveCreaturesToPlayer(PlayerID, Vec<CreatureID>),
  /// Remove a player from the game, allowing all of their creatures to be given to other players.
  UnregisterPlayer(PlayerID),
  /// Remove control of a creature from a player.
  RemoveCreaturesFromPlayer(PlayerID, Vec<CreatureID>),
  /// Move a player to a particular scene, so they only see what's happening in that scene.
  /// Note that this doesn't have any affect on a player's *characters*.
  /// The scene name can be None (null) to not show any scene to the player.
  SetPlayerScene(PlayerID, Option<SceneID>),

  /// Roll back to a specific snapshot + log index
  Rollback(usize, usize),
}

/// A representation of state change in a Creature. All change to a Creature happens via these
/// values. Note that these represent *concrete* changes to the Creature, which will have
/// deterministic results.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CreatureLog {
  Damage(HP, Vec<u8>),
  Heal(HP, Vec<u8>),
  GenerateEnergy(Energy),
  ReduceEnergy(Energy),
  ApplyCondition(ConditionID, ConditionDuration, Condition),
  DecrementConditionRemaining(ConditionID),
  RemoveCondition(ConditionID),
}

/// Representation of state changes in a Combat. See `CreatureLog`.
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum CombatLog {
  ConsumeMovement(Distance),
  ChangeCreatureInitiative(CreatureID, usize),
  EndTurn(CreatureID), // the end of this creature's turn
}

pub fn creature_logs_into_game_logs(cid: CreatureID, ls: Vec<CreatureLog>) -> Vec<GameLog> {
  ls.into_iter().map(|l| GameLog::CreatureLog(cid.clone(), l)).collect()
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum GameLog {
  // ** Abilities **
  SimpleAttributeCheckResult(CreatureID, AttrID, SkillLevel, bool),
  RandomAttributeCheckResult(CreatureID, AttrID, SkillLevel, u8, bool),

  // ** Folder Management **
  /// Create a folder, given segments leading to it.
  CreateFolder(FolderPath),
  RenameFolder(FolderPath, String),
  DeleteFolder(FolderPath),
  MoveFolderItem(FolderPath, FolderItemID, FolderPath),
  CreateNote(FolderPath, Note),
  EditNote(FolderPath, String, Note),
  DeleteNote(FolderPath, String),

  CreateScene(FolderPath, Scene),
  EditScene(Scene),
  DeleteScene(SceneID),
  CreateMap(FolderPath, Map),
  EditMap(Map),
  DeleteMap(MapID),
  CombatLog(CombatLog),
  /// A creature log wrapped in a game log.
  CreatureLog(CreatureID, CreatureLog),
  SetCreaturePos(SceneID, CreatureID, Point3),
  PathCreature(SceneID, CreatureID, Vec<Point3>),
  StartCombat(SceneID, Vec<CreatureID>),
  StopCombat,
  CreateCreature(FolderPath, Creature),
  EditCreature(Creature),
  DeleteCreature(CreatureID),
  AddCreatureToCombat(CreatureID),
  RemoveCreatureFromCombat(CreatureID),
  /// Indexes into snapshots and logs.
  Rollback(usize, usize),
}

pub fn combat_logs_into_game_logs(ls: Vec<CombatLog>) -> Vec<GameLog> {
  ls.into_iter().map(|l| GameLog::CombatLog(l)).collect()
}

/// An error in P&T.


error_chain! {
  types { GameError, GameErrorEnum, GameErrorResultExt; }

  links {
    FolderTreeError(FolderTreeError, FolderTreeErrorKind);
  }

  foreign_links {
    UUIDParseError(UuidParseError);
  }

  errors {
    AttributeNotFound(cid: CreatureID, attrid: AttrID) {
      description("A Creature does not have the supplied Attribute")
      display("The Creature with ID {} does not have the attribute {}", cid.to_string(), attrid.0)
    }
    CreatureAlreadyExists(cid: CreatureID) {
      description("A Creature with the given ID already exists")
      display("The creature with ID {} already exists", cid.to_string())
    }
    SceneAlreadyExists(scene: SceneID) {
      description("A scene already exists.")
      display("The scene {} already exists", scene.0)
    }
    SceneNotFound(scene: SceneID) {
      description("A scene wasn't found")
      display("The scene '{}' wasn't found", scene.0)
    }
    SceneInUse(scene: SceneID) {
      description("The scene can't be deleted because it's in use (likely because it's in combat).")
      display("The scene {} is in use (by combat, probably).", scene.0)
    }
    IDTooLong(id: String) {
      description("An identifier was too long.")
      display("The identifier '{}' is too long.", id)
    }
    ConditionNotFound(id: ConditionID) {
      description("A condition wasn't found.")
      display("The condition with ID {} wasn't found.", id)
    }
    InvalidCommand(cmd: GameCommand) {
      description("The supplied GameCommand is not valid in the current state.")
      display("Cannot process {:?} in this state.", cmd)
    }
    ClassNotFound(cls: String) {
      description("A class wasn't found.")
      display("The class {} was not found.", cls)
    }
    NoAbility(abid: AbilityID) {
      description("An ability wasn't found.")
      display("The ability with ID {} wasn't found.", abid.to_string())
    }
    CombatMustHaveCreatures {
      description("Combat can't be started without creatures.")
      display("Creatures must be supplied when starting a combat.")
    }
    CreatureLacksAbility(cid: CreatureID, abid: AbilityID) {
      description("A creature cannot use the supplied ability.")
      display("The creature with ID {} does not have the ability {}", cid.to_string(), abid.to_string())
    }
    CreatureNotFound(id: String) {
      description("A creature with the supplied ID could not be found.")
      display("The creature with ID {} could not be found.", id)
    }
    InvalidTarget(cid: CreatureID) {
      description("The specified creature is not a valid target.")
      display("Creature with ID {} is not a valid target.", cid.to_string())
    }
    InvalidTargetForTargetSpec(tspec: TargetSpec, dtarget: DecidedTarget) {
      description("The supplied DecidedTarget is not valid for the TargetSpec in use.")
      display("DecidedTarget {:?} is not valid for TargetSpec {:?}.", dtarget, tspec)
    }
    CreatureOutOfRange(cid: CreatureID) {
      description("The specified creature is out of range.")
      display("Creature {} is out of range.", cid.to_string())
    }
    BuggyProgram(msg: String) {
      description("There was an internal error that is caused by a broken assumption, indicating that this software is garbage.")
      display("There's a bug in the program: {}", msg)
    }
    NotInCombat {
      description("There is currently no combat when trying to do something combat-specific.")
      display("There is currently no combat.")
    }
    AlreadyInCombat(cid: CreatureID) {
      description("The specified creature is already in combat.")
      display("Creature {} is already in combat.", cid.to_string())
    }
    CannotMove(cid: CreatureID) {
      description("A creature cannot move.")
      display("Creature {} cannot be moved.", cid.to_string())
    }
    CannotAct(cid: CreatureID) {
      description("A creature cannot act.")
      display("Creature {} cannot act.", cid.to_string())
    }
    NoPathFound {
      description("A path can't be found.")
      display("A path can't be found.")
    }
    StepTooBig(from: Point3, to: Point3) {
      description("A step from one point to another is too large.")
      display("Can't step from {:?} to {:?}", from, to)
    }
    MapNotFound(map: MapID) {
      description("The specified map was not found.")
      display("Couldn't find map {}", map.0)
    }
    MapAlreadyExists(map: MapID) {
      description("The specified map already exists.")
      display("Map {} already exists", map.0)
    }
    MapInUse(map: MapID, scenes: Vec<SceneID>) {
      description("A map can't be deleted because it is being referenced by one or more scenes.")
      display("Map {} is in use by scenes {:?}", map.0, scenes)
    }
    NotEnoughEnergy(nrg: Energy) {
      description("There is not enough energy to do something.")
      display("Not enough energy: {:?}", nrg)
    }
    PlayerAlreadyExists(pid: PlayerID) {
      description("The specified player ID is already registered.")
      display("Player ID {} is already registered.", pid.0)
    }
    PlayerNotFound(pid: PlayerID) {
      description("The specified player was not found.")
      display("Player ID {} was not found.", pid.0)
    }
    PlayerDoesntControlCreature(pid: PlayerID, cid: CreatureID) {
      description("The specified creature is not controlled by the current player.")
      display("Player ID {} does not control creature {}.", pid.0, cid.to_string())
    }
    HistoryNotFound(snap_idx: usize, log_idx: usize) {
      description("The requested history item was not found.")
      display("Couldn't find history item at snapshot {} log item {}", snap_idx, log_idx)
    }
    InitiativeOutOfBounds(idx: usize) {
      description("The initiative index is out of bound.")
      display("Initiative index {} is out of bounds.", idx)
    }
    FolderNotEmpty(path: FolderPath) {
      description("The user attempted to delete a folder when it wasn't empty.")
      display("The folder {} is not empty", path.to_string())
    }
    NoteNotFound(path: FolderPath, name: String) {
      description("A note couldn't be found.")
      display("The note in '{}' named '{}' could not be found.", path.to_string(), name)
    }
    CannotLinkNotes(path: FolderPath, name: String) {
      description("Notes can't be linked or unlinked.")
      display("Notes can't be linked or unlinked. '{}' / '{}'", path.to_string(), name)
    }
  }
}

/// A specification for what kind of targeting an ability uses. i.e., this describes the rules of
/// targeting for an ability definition, not the choice of a specific target during gameplay. See
/// `DecidedTarget` for that. The parameters of these variants indicate things like how far an
/// arrow can travel or what the radius of an AoE is.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TargetSpec {
  Melee,
  Range(Distance), /* CircleWithinRange(Distance, u8), // radius
                    * Cone(Distance, u8), // radians of angle of cone (should this be steradians? is it the same?)
                    * Line(Distance),
                    * LineToFirstHit(), */
  Actor,
}

/// The target of an ability, as chosen at play-time by a player. Generally this falls into
/// "specific creature" targeting (`Melee` and `Ranged`) and "aoe" targeting (the others). The
/// paremeters of these variants indicate the specific target creature or point that is being
/// targeted by the player.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DecidedTarget {
  Melee(CreatureID),
  // MeleeArea(Point3) // creatures can try attacking a square when they can't directly target a
  // creature -- for example if they think an invisible creature is in the
  // square. This could also be useful for things like breaking down doors.
  Range(CreatureID), /* CircleWithinRange(Point3),
                      * Cone(Angle2d),
                      * Line(Point3),
                      * LineToFirstHit(Point3), */
  Actor,
}

/// Potential targets for an ability.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PotentialTarget {
  CreatureID(CreatureID),
  Point(Point3),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Ability {
  pub name: String,
  pub target: TargetSpec,
  pub cost: Energy,
  pub effects: Vec<Effect>,
  pub usable_ooc: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Effect {
  // Interrupt,
  // Resurrect,
  ApplyCondition(ConditionDuration, Condition),
  Heal(Dice),
  Damage(Dice),
  MultiEffect(Vec<Effect>),
  GenerateEnergy(Energy),
}


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Condition {
  RecurringEffect(Box<Effect>),
  Dead,
  Incapacitated,
  AddDamageBuff(HP),
  DoubleMaxMovement,
  ActivateAbility(AbilityID),
}

impl Condition {
  pub fn apply(&self, duration: ConditionDuration) -> AppliedCondition {
    AppliedCondition {
      remaining: duration,
      condition: self.clone(),
    }
  }
}

/// Serializes as either "Interminate" or {"Duration": 0}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConditionDuration {
  Interminate,
  Duration(u8),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AppliedCondition {
  pub remaining: ConditionDuration,
  pub condition: Condition,
}


#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct AbilityStatus {
  pub ability_id: AbilityID,
  pub cooldown: u8,
}

/// A creature class, e.g. rogue, mage, warrior
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Class {
  /// A list of abilities that this class can use.
  pub abilities: Vec<AbilityID>,
  /// A list of conditions which will be *permanently* applied to any creature in this class.
  pub conditions: Vec<Condition>,
  /// An SVG-compatible color specifier
  pub color: String,
}

/// A specification for creating a new creature.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct CreatureCreation {
  pub name: String,
  pub class: String,
  pub portrait_url: String,
}

/// A Creature.
///
/// A very important thing about how we deal with creatures is that whenever we change
/// a creature, we get back both a new creature *and* a log of all things that happened to that
/// creature. That log is deterministic and complete enough for us to replay it on a snapshot of a
/// creature and get an identical creature.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Creature {
  pub id: CreatureID,
  pub name: String,
  pub speed: Distance,
  pub max_energy: Energy,
  pub cur_energy: Energy,
  pub abilities: Vec<AbilityStatus>,
  pub class: String,
  pub max_health: HP,
  pub cur_health: HP,
  pub conditions: HashMap<ConditionID, AppliedCondition>,
  pub note: String,
  pub portrait_url: String,
  pub attributes: HashMap<AttrID, SkillLevel>,
}

#[derive(Clone, Hash, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct AttrID(pub String);

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Combat {
  pub scene: SceneID,
  pub creatures: nonempty::NonEmptyWithCursor<CreatureID>,
  pub movement_used: Distance,
}

impl DeriveKey for Creature {
  type KeyType = CreatureID;
  fn derive_key(&self) -> CreatureID {
    self.id
  }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Game {
  pub current_combat: Option<Combat>,
  pub abilities: HashMap<AbilityID, Ability>,
  pub creatures: HashMap<CreatureID, Creature>,
  pub maps: IndexedHashMap<Map>,
  pub classes: HashMap<String, Class>,
  pub tile_system: TileSystem,
  pub scenes: IndexedHashMap<Scene>,
  pub campaign: FolderTree<Folder>,
}

/// A data structure maintaining state for the whole app. It keeps track of the history of the
/// whole game, and exposes the top-level methods that run simulations on the game.
#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct App {
  pub current_game: Game,
  pub snapshots: VecDeque<(Game, Vec<GameLog>)>,
  pub players: IndexedHashMap<Player>,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Player {
  pub player_id: PlayerID,
  pub scene: Option<SceneID>,
  pub creatures: HashSet<CreatureID>,
}

impl DeriveKey for Player {
  type KeyType = PlayerID;
  fn derive_key(&self) -> PlayerID {
    self.player_id.clone()
  }
}

impl Player {
  pub fn new(name: PlayerID) -> Player {
    Player {
      player_id: name,
      scene: None,
      creatures: HashSet::new(),
    }
  }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct SceneCreation {
  pub name: String,
  pub map: MapID,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Scene {
  pub id: SceneID,
  pub name: String,
  pub map: MapID,
  pub creatures: HashMap<CreatureID, (Point3, Visibility)>,
  pub attribute_checks: HashMap<String, SkillCheck>,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct SkillCheck {
  pub random: bool,
  pub attr: AttrID,
  pub target: SkillLevel
}

impl DeriveKey for Scene {
  type KeyType = SceneID;
  fn derive_key(&self) -> SceneID {
    self.id
  }
}

impl Scene {
  pub fn create(creation: SceneCreation) -> Scene {
    Scene {
      id: SceneID::new(),
      name: creation.name,
      map: creation.map,
      creatures: HashMap::new(),
      attribute_checks: HashMap::new(),
    }
  }
  pub fn get_pos(&self, creature_id: CreatureID) -> Result<Point3, GameError> {
    self.creatures
      .get(&creature_id)
      .map(|x| x.0)
      .ok_or_else(|| GameErrorEnum::CreatureNotFound(creature_id.to_string()).into())
  }

  pub fn set_pos(&self, cid: CreatureID, pt: Point3) -> Result<Scene, GameError> {
    let mut new = self.clone();
    {
      let data = new.creatures
        .get_mut(&cid)
        .ok_or_else(|| GameErrorEnum::CreatureNotFound(cid.to_string()))?;
      data.0 = pt;
    }
    Ok(new)
  }
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Visibility {
  GMOnly,
  AllPlayers,
}


#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct MapCreation {
  pub name: String,
  pub terrain: Vec<Point3>,
}

#[derive(Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Map {
  pub id: MapID,
  pub name: String,
  pub terrain: Vec<Point3>,
  pub specials: Vec<(Point3, Color, String, Visibility)>,
}

impl Map {
  pub fn create(c: MapCreation) -> Map {
    Map::new(c.name, c.terrain)
  }

  pub fn new(name: String, terrain: Vec<Point3>) -> Map {
    Map {
      id: MapID::new(),
      name: name,
      terrain: terrain,
      specials: vec![],
    }
  }

  pub fn is_open(&self, pt: &Point3) -> bool {
    self.terrain.contains(pt)
  }
}


impl DeriveKey for Map {
  type KeyType = MapID;
  fn derive_key(&self) -> MapID {
    self.id
  }
}


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicCombat<'game> {
  pub scene: &'game Scene,
  pub map: &'game Map,
  pub combat: &'game Combat,
  pub game: &'game Game,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DynamicCreature<'creature, 'game: 'creature> {
  pub creature: &'creature Creature,
  pub game: &'game Game,
  pub class: &'game Class,
}

/// A newtype wrapper over App that has a special Serialize implementation, which includes extra
/// data dynamically as a convenience for the client.
pub struct RPIApp<'a>(pub &'a App);
/// Like RPIApp for Game.
pub struct RPIGame<'a>(pub &'a Game);

impl<'a> ser::Serialize for RPIApp<'a> {
  fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let mut str = serializer.serialize_struct("App", 3)?;
    let app = self.0;
    str.serialize_field("current_game", &RPIGame(&app.current_game))?;
    str.serialize_field("snapshots", &app.snapshots)?;
    str.serialize_field("players", &app.players)?;
    str.end()
  }
}

impl<'a> ser::Serialize for RPIGame<'a> {
  fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let mut str = serializer.serialize_struct("Game", 9)?;
    let game = self.0;

    str.serialize_field("current_combat", &game.current_combat)?;
    str.serialize_field("abilities", &game.abilities)?;
    str.serialize_field("creatures", &game.creatures().map_err(|e| S::Error::custom("Oh no!"))?)?;
    str.serialize_field("maps", &game.maps)?;
    str.serialize_field("classes", &game.classes)?;
    str.serialize_field("tile_system", &game.tile_system)?;
    str.serialize_field("scenes", &game.scenes)?;
    str.serialize_field("campaign", &game.campaign)?;
    str.end()
  }
}

impl<'creature, 'game: 'creature> ser::Serialize for DynamicCreature<'creature, 'game> {
  fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    let mut str = serializer.serialize_struct("Creature", 17)?;
    let creat = &self.creature;
    str.serialize_field("id", &creat.id)?;
    str.serialize_field("name", &creat.name)?;
    str.serialize_field("note", &creat.note)?;
    str.serialize_field("portrait_url", &creat.portrait_url)?;
    str.serialize_field("speed", &self.speed())?;
    str.serialize_field("max_energy", &creat.max_energy)?;
    str.serialize_field("cur_energy", &creat.cur_energy)?;
    str.serialize_field("abilities", &self.ability_statuses())?;
    str.serialize_field("class", &creat.class)?;
    str.serialize_field("max_health", &creat.max_health)?;
    str.serialize_field("cur_health", &creat.cur_health)?;
    str.serialize_field("conditions", &creat.conditions)?;
    str.serialize_field("attributes", &creat.attributes)?;
    str.serialize_field("can_act", &self.can_act())?;
    str.serialize_field("can_move", &self.can_move())?;
    str.end()
  }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum TileSystem {
  /// Square grid with diagonal movement costing 1.41
  Realistic,
  /// Square grid with diagonal movement costing 1
  DnD,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Note {
  pub name: String,
  pub content: String,
}

impl DeriveKey for Note {
  type KeyType = String;
  fn derive_key(&self) -> String {
    self.name.clone()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct Folder {
  pub scenes: HashSet<SceneID>,
  pub creatures: HashSet<CreatureID>,
  pub notes: IndexedHashMap<Note>,
  pub maps: HashSet<MapID>,
}

impl Folder {
  pub fn new() -> Folder {
    Folder {
      scenes: HashSet::new(),
      creatures: HashSet::new(),
      notes: IndexedHashMap::new(),
      maps: HashSet::new(),
    }
  }
}

#[cfg(test)]
pub mod test {
  use types::*;
  use std::iter::FromIterator;

  use serde_yaml;
  use serde_json;

  pub fn t_rogue_creation(name: &str) -> CreatureCreation {
    CreatureCreation {
      name: name.to_string(),
      class: "rogue".to_string(),
      portrait_url: "".to_string(),
    }
  }

  pub fn t_cleric_creation(name: &str) -> CreatureCreation {
    CreatureCreation {
      name: name.to_string(),
      class: "cleric".to_string(),
      portrait_url: "".to_string(),
    }
  }

  pub fn t_ranger_creation(name: &str) -> CreatureCreation {
    CreatureCreation {
      name: name.to_string(),
      class: "ranger".to_string(),
      portrait_url: "".to_string(),
    }
  }

  pub fn cid_cleric() -> CreatureID {
    CreatureID::from_str("00000000-0000-0000-0000-000000000000").unwrap()
  }
  pub fn cid_ranger() -> CreatureID {
    CreatureID::from_str("00000000-0000-0000-0000-000000000001").unwrap()
  }
  pub fn cid_rogue() -> CreatureID {
    CreatureID::from_str("00000000-0000-0000-0000-000000000002").unwrap()
  }

  pub fn t_scene_id() -> SceneID {
    SceneID::from_str("00000000-0000-0000-0000-000000000003").unwrap()
  }

  pub fn t_map_id() -> MapID {
    MapID::from_str("00000000-0000-0000-0000-000000000004").unwrap()
  }

  pub fn app_cond(c: Condition, r: ConditionDuration) -> AppliedCondition {
    AppliedCondition {
      condition: c,
      remaining: r,
    }
  }

  pub fn t_punch() -> Ability {
    Ability {
      name: "Punch".to_string(),
      target: TargetSpec::Melee,
      cost: Energy(0),
      usable_ooc: true,
      effects: vec![Effect::Damage(Dice::flat(3))],
    }
  }

  pub fn t_shoot() -> Ability {
    Ability {
      name: "Shoot".to_string(),
      target: TargetSpec::Range(Distance::from_meters(5.0)),
      cost: Energy(0),
      usable_ooc: true,
      effects: vec![Effect::Damage(Dice::flat(3))],
    }
  }

  pub fn t_heal() -> Ability {
    Ability {
      name: "Heal".to_string(),
      target: TargetSpec::Range(Distance::from_meters(5.0)),
      cost: Energy(0),
      usable_ooc: true,
      effects: vec![Effect::Heal(Dice::flat(3))],
    }
  }

  pub fn t_abilities() -> HashMap<AbilityID, Ability> {
    let punch = t_punch();
    let shoot = t_shoot();
    let heal = t_heal();
    HashMap::from_iter(vec![(abid("punch"), punch), (abid("shoot"), shoot), (abid("heal"), heal)])
  }

  #[test]
  fn serde_ids() {
    let id = abid("foobar");
    assert_eq!(serde_yaml::to_string(&id).unwrap(), "---\nfoobar");
  }

  #[test]
  fn serde_condition_duration() {
    let cd = ConditionDuration::Interminate;
    assert_eq!(serde_json::to_string(&cd).unwrap(), "\"Interminate\"");
    let cd = ConditionDuration::Duration(3);
    assert_eq!(serde_json::to_string(&cd).unwrap(), "{\"Duration\":3}");
  }
}
