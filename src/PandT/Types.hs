{-# LANGUAGE DataKinds #-}
{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE TemplateHaskell #-}

module PandT.Types where

import Control.Lens ((^.), over, makeLenses)
import Data.Text (Text)
import Data.Foldable (foldl')

data Intensity = Low | Medium | High
    deriving (Show, Eq, Ord)

newtype DamageIntensity = DamageIntensity Intensity
    deriving (Show, Eq, Ord)

newtype Stamina = Stamina Intensity
    deriving (Show, Eq, Ord)

newtype Range = Range Int
    deriving (Show, Eq, Ord)

newtype Radius = Radius Int
    deriving (Show, Eq, Ord)

newtype Duration = Duration Int
    deriving (Show, Eq, Ord)

newtype Period = Period Int
    deriving (Show, Eq, Ord)

newtype CastTime = CastTime Int
    deriving (Show, Eq, Ord)

newtype Cooldown = Cooldown Int
    deriving (Show, Eq, Ord)

newtype Health = Health Int
    deriving (Show, Eq, Ord)

data Resource = Mana Int | Energy Int
    deriving (Show, Eq)


data ConditionDuration -- this could have a reasonable Ord instance
    = TimedCondition Duration
    | UnlimitedDuration
    deriving (Show, Eq)

data ConditionValue
    = RecurringEffect Period Effect
    | DamageAbsorb DamageIntensity
    | DamageBuff DamageIntensity
    deriving (Show, Eq)

data Condition
    = Condition
    { _conditionName :: Text
    , _conditionDuration :: ConditionDuration
    , _conditionValue :: ConditionValue
    } deriving (Show, Eq)


data Effect
    = Interrupt
    | ApplyCondition Condition
    | Heal DamageIntensity
    | Damage DamageIntensity
    deriving (Show, Eq)


data TargetSystem
    = TargetCreature Range
    | TargetCircle Range Radius
    | TargetLineFromSource Range
    deriving (Show, Eq)


data Ability = Ability
    { _name :: Text
    , _cost :: Resource
    , _effects :: [Effect]
    , _target :: TargetSystem
    , _castTime :: CastTime
    , _cooldown :: Cooldown
    }
    deriving (Show, Eq)

makeLenses ''Ability

data Creature = Creature
    { _conditions :: [Condition]
    , _resource :: Resource
    , _stamina :: Stamina
    , _health :: Health
    , _abilities :: [Ability]
    , _casting :: Maybe Ability
    }
    deriving (Show, Eq)

makeLenses ''Creature

staminaToHealth :: Stamina -> Health
staminaToHealth (Stamina High) = Health 100
staminaToHealth (Stamina Medium) = Health 50
staminaToHealth (Stamina Low) = Health 25

damageToHealthVal :: DamageIntensity -> Int
damageToHealthVal (DamageIntensity High) = 50
damageToHealthVal (DamageIntensity Medium) = 25
damageToHealthVal (DamageIntensity Low) = 10

healthMinusDamage :: Health -> DamageIntensity -> Health
healthMinusDamage (Health healthVal) dmg = Health (healthVal - (damageToHealthVal dmg))

healthPlusDamage :: Health -> DamageIntensity -> Health
healthPlusDamage (Health healthVal) dmg = Health (healthVal + (damageToHealthVal dmg))

makeCreature :: Resource -> Stamina -> [Ability] -> Creature
makeCreature res sta creatAbilities = Creature
    { _conditions=[]
    , _resource=res
    , _stamina=sta
    , _health=staminaToHealth sta
    , _abilities=creatAbilities
    , _casting=Nothing}

makeDotEffect :: Text -> Intensity -> ConditionDuration -> Period -> Effect
makeDotEffect newConditionName int dur per
    = ApplyCondition
        (Condition
            { _conditionName=newConditionName
            , _conditionValue=RecurringEffect per (Damage (DamageIntensity int))
            , _conditionDuration=dur})

bleed :: Effect
bleed = makeDotEffect "Bleeding" Medium (TimedCondition (Duration 2)) (Period 1)

stab :: Ability
stab = Ability
    { _name="Stab"
    , _cost=Energy 10
    , _effects=[Damage (DamageIntensity Medium), bleed]
    , _target=TargetCreature (Range 1)
    , _castTime = CastTime 0
    , _cooldown = Cooldown 0
    }

creat :: Creature
creat = makeCreature (Energy 100) (Stamina High) [stab]

applyEffect :: Creature -> Effect -> Creature
applyEffect creature effect = go effect
    where
        go (ApplyCondition condition) = over conditions (condition:) creature
        go (Damage amt) = over health (flip healthMinusDamage amt) creature
        go (Heal amt) = over health (flip healthPlusDamage amt) creature

applyAbility :: Ability -> Creature -> Creature
applyAbility abil creatu
    = foldl' applyEffect creatu (abil^.effects)

dotted :: Creature
dotted = applyEffect creat bleed

damaged :: Creature
damaged = applyEffect creat (Damage (DamageIntensity Medium))

healed :: Creature
healed = applyEffect damaged (Heal (DamageIntensity Low))

{-
abilities I want
- basic damage attack
- damage + dot
- damage + heal a target. target both distinctly!
- heal
- heal over time
- buff to damage
- debuff to damage


-}
