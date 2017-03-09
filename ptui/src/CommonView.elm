module CommonView exposing
  (visibleCreatures, creatureCard, oocActionBar, combatActionBar, moveOOCButton, mapControls)

import Dict

import Css as S
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)

import Model as M
import Types as T
import Grid

import Elements exposing (..)

s = Elements.s -- to disambiguate `s`, which Html also exports
button = Elements.button

visibleCreatures : M.Model -> T.Game -> List Grid.MapCreature
visibleCreatures model game =
  let mapInfo creature =
    Maybe.map (\class -> {creature = creature, highlight = False, movable = Nothing, class = class})
              (Dict.get creature.class game.classes)
  in
    case game.current_combat of
      Just combat ->
        if model.showOOC
        then List.filterMap mapInfo <| combat.creatures.data ++ (Dict.values game.creatures)
        else List.filterMap mapInfo combat.creatures.data
      Nothing -> (List.filterMap mapInfo (Dict.values game.creatures))

creatureCard : List (Html M.Msg) -> M.Model -> T.Creature -> Html M.Msg
creatureCard extras model creature =
  let cellStyles color =
        [s [ plainBorder
           , S.backgroundColor color
           , S.borderRadius (S.px 10)
           , S.padding (S.px 3)]]
  in
    vabox
      [s [plainBorder, S.width (S.px 300), S.height (S.px 100), S.borderRadius (S.px 10), S.padding (S.px 3)]]
      <| 
      [ hbox [strong [] [text creature.name ], classIcon creature]
      , hbox [
        --  div (cellStyles (S.rgb 144 238 144))
        --         [text <| (toString creature.cur_health) ++ "/" ++ (toString creature.max_health)]
            div (cellStyles (S.rgb 0 255 255))
                [text <| (toString creature.cur_energy) ++ "/" ++ (toString creature.max_energy)]
            , div (cellStyles (S.rgb 255 255 255))
                [text <| (toString creature.pos.x) ++ ", " ++ (toString creature.pos.y)]
            ]
      -- , hbox [ div (cellStyles (S.rgb 255 255 255)) [text "💪 10"]
      --        , div (cellStyles (S.rgb 255 255 255)) [text "🛡️ 10"]
      --        , div (cellStyles (S.rgb 255 255 255)) [text "🏃 10"]]
      , hbox (List.map conditionIcon (Dict.values creature.conditions))
      ] ++ extras

classIcon : T.Creature -> Html M.Msg
classIcon creature =
  case creature.class of
    "cleric" -> text "💉"
    "rogue" -> text "🗡️"
    "ranger" -> text "🏹"
    "creature" -> text "🏃"
    _ -> text ""

conditionIcon : T.AppliedCondition -> Html M.Msg
conditionIcon ac = case ac.condition of 
  T.RecurringEffect eff -> text (toString eff)
  T.Dead -> text "💀"
  T.Incapacitated -> text "😞"
  T.AddDamageBuff dmg -> text "😈"
  T.DoubleMaxMovement -> text "🏃"
  T.ActivateAbility abid -> hbox [text <| "ACTIVE: " ++ abid, durationEl ac.remaining]

durationEl : T.ConditionDuration -> Html M.Msg
durationEl condDur = case condDur of
  T.Interminate -> text "∞"
  T.Duration n -> text <| "(" ++ toString n ++ ")"


baseActionBar : Bool -> T.Game -> T.Creature -> List (Html M.Msg)
baseActionBar inCombat game creature =
  let abinfo abstatus = Maybe.andThen (\ability -> if ability.usable_ooc || inCombat then Just (abstatus.ability_id, ability) else Nothing)
                                  (Dict.get abstatus.ability_id game.abilities)
      abilities = List.filterMap abinfo creature.abilities
  in (List.map (abilityButton creature) abilities)

oocActionBar = baseActionBar False

combatActionBar : T.Game -> T.Combat -> T.Creature -> Html M.Msg
combatActionBar game combat creature =
  habox [s [S.flexWrap S.wrap]] ([doneButton creature] ++ [moveButton combat creature] ++  baseActionBar True game creature)

abilityButton : T.Creature -> (T.AbilityID, T.Ability) -> Html M.Msg
abilityButton creature (abid, ability) =
  actionButton
    [ onClick (M.SelectAbility creature.id abid)
    , disabled (not creature.can_act)]
    [text ability.name]

actionButton : List (Attribute msg) -> List (Html msg) -> Html msg
actionButton attrs children =
  button ([s [S.height (S.px 50), S.width (S.px 100)]] ++ attrs) children

moveOOCButton : T.Creature -> Html M.Msg
moveOOCButton creature =
  button [ onClick (M.GetMovementOptions creature)
         , disabled (not creature.can_move)]
         [text "Move"]

doneButton : T.Creature -> Html M.Msg
doneButton creature =
  actionButton [onClick (M.SendCommand T.Done)] [text "Done"]

moveButton : T.Combat -> T.Creature -> Html M.Msg
moveButton combat creature =
  let movement_left = creature.speed - combat.movement_used
  in actionButton [ onClick M.GetCombatMovementOptions
            , disabled (not creature.can_move) ]
            [text (String.join "" ["Move (", toString (movement_left // 100), ")"])]

{-| The map controls: panning and zooming buttons
-}
mapControls : Html M.Msg
mapControls =
  vabox
    [s [ S.backgroundColor (S.rgb 230 230 230)]]
    [ hbox [ sqButton 40 [onClick (M.MapZoom M.Out)] [text "-"]
           , sqButton 40 [onClick (M.MapZoom M.In)] [text "+"]
           ]
    , vbox
        [ sqButton 40 [s [S.alignSelf S.center], onClick (M.MapPan M.Up)] [text "^"]
        , hbox
            [ sqButton 40 [s [S.flexGrow (S.int 1)], onClick (M.MapPan M.Left)] [text "<"]
            , sqButton 40 [s [S.flexGrow (S.int 1)], onClick (M.MapPan M.Right)] [text ">"]
            ]
        , sqButton 40 [s [S.alignSelf S.center], onClick (M.MapPan M.Down)] [text "v"]
        ]
    ]
