{-# LANGUAGE RecordWildCards #-}

-- | Text renderers for stuff that needs rendered. Much of this should be obsolete in the long term:
-- we should just be serializing data to JSON and have the client render it, but some bits might
-- remain.

module PandT.Render where

import PandT.Prelude
import PandT.Types


renderConditionDef :: ConditionDef -> Text
renderConditionDef (ConditionDef meta condc) =
    meta^.conditionName ++ " (" ++ (renderConditionCaseName condc) ++ ", " ++ (renderConditionDuration (meta^.conditionDuration)) ++ ")"

renderConditionDuration :: ConditionDuration -> Text
renderConditionDuration UnlimitedDuration = "unlimited"
renderConditionDuration (TimedCondition (Duration n)) = (tshow n) ++ " rounds"

renderConditionCaseName :: ConditionC -> Text
renderConditionCaseName = n
    where
        n (RecurringEffectC _) = "Recurring Effect"
        n (DamageAbsorbC _) = "Damage Absorb"
        n (DamageIncreaseC _) = "Damage Increase"
        n (DamageDecreaseC _) = "Damage Decrease"
        n (IncapacitatedC _)  = "Incapacitation"
        n (DeadC _) = "Death"

renderEffectOccurrence :: EffectOccurrence -> Text
renderEffectOccurrence = unlines . (map go)
    where
        go (Interrupt, cname) = "interrupting " ++ cname
        go ((Heal int), cname) = tshow int ++ " healing to " ++ cname
        go ((Damage int), cname) = tshow int ++ " damage to " ++ cname
        go ((MultiEffect eff1 eff2), cname) = renderEffectOccurrence [(eff1, cname), (eff2, cname)]
        go ((ApplyCondition cdef), cname) = "Applying condition " ++ (renderConditionDef cdef) ++ " to " ++ cname

renderCombatEvent :: CombatEvent -> Text
renderCombatEvent (AbilityUsed abil source occurrences) =
    source ++ " used " ++ (abil^.abilityName) ++ ", causing:\n" ++ (renderEffectOccurrence occurrences) ++ "."
renderCombatEvent (CreatureTurnStarted name) = name ++ "'s turn stared."
renderCombatEvent (AbilityStartCast cname ab) = cname ++ " started casting " ++ (ab^.abilityName) ++ "."
renderCombatEvent (RecurringEffectOccurred source target occurrences) =
    source ++ "'s recurring effect ticked on " ++ target ++ ", causing:\n" ++ (renderEffectOccurrence occurrences) ++ "."
renderCombatEvent (SkippedIncapacitatedCreatureTurn creatName) = creatName ++ " is incapacitated."
renderCombatEvent (SkippedTurn creatName) = creatName ++ " skipped their turn."
renderCombatEvent (CanceledCast creatName ability) = creatName ++ " stopped casting " ++ (ability^.abilityName)

class RenderState a where
    renderState :: a -> Text

instance RenderState PlayerChoosingAbility where
    renderState PlayerChoosingAbility = "Choosing ability"
instance RenderState PlayerIncapacitated where
    renderState PlayerIncapacitated = "Incapacitated"
instance RenderState PlayerChoosingTargets where
    renderState (PlayerChoosingTargets ability) = "Choosing targets for " ++ _abilityName ability
instance RenderState GMVettingAction where
    renderState (GMVettingAction ability targets) = "GM vetting action for " ++ _abilityName ability ++ " -> " ++ tshow targets
instance RenderState PlayerCasting where
    renderState PlayerCasting = "Casting"
instance RenderState PlayerFinishingCast where
    renderState PlayerFinishingCast = "Finishing cast"

renderCreatureStatus :: Creature -> Text
renderCreatureStatus creature =
    line
    where
        hp = tshow $ creature^.health
        conds = concat (intersperse "; " (map renderAppliedCondition (creature^.conditions)))
        castSumm = case creature^.casting of
            Nothing -> ""
            Just (ability, (Duration duration)) ->
                "(casting " ++ (ability^.abilityName) ++ " for " ++ (tshow duration) ++ " more rounds)"
        line = unwords [creature^.creatureName, hp, castSumm, conds]

renderAppliedCondition :: AppliedCondition -> Text
renderAppliedCondition (AppliedCondition originCreatureName duration meta _) =
    originCreatureName ++ "'s " ++ (meta^.conditionName) ++ " (" ++ (renderConditionDuration duration) ++ ")"


renderInitiative :: Game a -> Text
renderInitiative game
    = let
        currentName = game^.currentCreatureName
        creature name = view (creaturesInPlay . at name) game
        pfx name = if name == currentName then "*" else " "
        statusLine name = unwords.toList $ renderCreatureStatus <$> creature name
        rend name = pfx name ++ statusLine name
    in
        unlines $ map rend (_initiative game)

render :: RenderState a => Game a -> Text
render game@(Game {..}) = unlines
    [ "# Game"
    , "Current creature: " ++ " (" ++ _currentCreatureName ++ ") "
    , renderState _state
    , renderInitiative game
    ]
