module View exposing (..)

import Array
import Dict
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Set

import Model as M
import Types as T
import Grid
import Update as U
import Elements exposing (..)
import GMView

-- TODO: delete these once we refactor player code out
import CommonView exposing (..)

import Css as S

s = Elements.s -- to disambiguate `s`, which Html also exports
button = Elements.button

gmView : M.Model -> Html M.Msg
gmView model = vbox
  [ case model.app of
      Just app -> GMView.viewGame model app
      Nothing -> text "No app yet. Maybe reload."
  , hbox [text "Last error:", pre [] [text model.error]]
  ]

oocToggler : M.Model -> Html M.Msg
oocToggler model =
  hbox [text "Show Out-of-Combat creatures: "
       , input [type_ "checkbox", checked model.showOOC, onClick M.ToggleShowOOC] []
       ]

editMapButton : Html M.Msg
editMapButton = button [onClick M.StartEditingMap] [text "Edit this map"]

playerView : M.Model -> Html M.Msg
playerView model = vbox
  [ case model.app of
      Just app ->
        let vGame = case model.playerID of
              Just playerID ->
                if T.playerIsRegistered app playerID
                then playerViewGame model app (T.getPlayerCreatures app playerID)
                else registerForm model
              Nothing -> registerForm model
        in vbox [vGame, playerList (always []) app.players]
      Nothing -> text "No app yet. Maybe reload."
  , hbox [text "Last error:", pre [] [text model.error]]
  ]

registerForm : M.Model -> Html M.Msg
registerForm model =
  hbox [ input [type_ "text", placeholder "player ID", onInput M.SetPlayerID ] []
       , button [onClick M.RegisterPlayer] [text "Register Player"]]

playerViewGame : M.Model -> T.App -> List T.Creature -> Html M.Msg
playerViewGame model app creatures = 
  let game = app.current_game in
  hbox
  [ case model.moving of
      Just movementRequest ->
        let {max_distance, movement_options, ooc_creature} = movementRequest in
        case ooc_creature of
          Just oocMovingCreature ->
            if List.member oocMovingCreature creatures
            then
              -- one of my characters is moving; render the movement map
              Grid.movementMap model (M.PathCreature oocMovingCreature.id) movementRequest
                               False model.currentMap oocMovingCreature (visibleCreatures model game)
            else
              -- someone else is moving (TODO: render a non-interactive movement map to show what they're doing)
              playerGrid model game creatures
          Nothing -> -- we're moving in-combat.
            case Maybe.map T.combatCreature game.current_combat of
                Just creature ->
                  let highlightMover mapc =
                        if creature.id == mapc.creature.id
                        then {mapc | highlight = True}
                        else mapc
                      vCreatures = List.map highlightMover (visibleCreatures model game)
                  in Grid.movementMap model M.PathCurrentCombatCreature movementRequest
                                   False model.currentMap creature vCreatures
                Nothing -> playerGrid model game creatures
      Nothing -> playerGrid model game creatures
  ]
  -- TODO: a read-only "creatures nearby" list without details

playerCombatArea : M.Model -> T.Game -> T.Combat -> List T.Creature -> Html M.Msg
playerCombatArea model game combat creatures =
  let currentCreature = T.combatCreature combat
      bar = if List.member currentCreature creatures
            then habox [s [S.flexWrap S.wrap]]
                       [ div [s [S.width (S.px 100)]] [strong [] [text currentCreature.name]]
                       , combatActionBar game combat currentCreature
                       ]
            else hbox [text "Current creature:", text currentCreature.id]
      selector = case model.selectedAbility of
                    Just (cid, abid) -> 
                          if T.isCreatureInCombat game cid
                          then [targetSelector model game M.CombatAct abid]
                          else []
                    Nothing -> []
      initiativeList = combatantList False model game combat
  in vbox <| [bar] ++ selector ++ [initiativeList]


-- render the grid and some OOC movement buttons for the given creatures
playerGrid : M.Model -> T.Game -> List T.Creature -> Html M.Msg
playerGrid model game myCreatures =
  let moveOOCButton creature =
        button [ onClick (M.GetMovementOptions creature)
               , disabled (not creature.can_move)]
               [text "Move"]
      bar creature =
        if T.isCreatureInCombat game creature.id
        then Nothing
        else Just <| hbox <| [creatureCard [] model creature, moveOOCButton creature] ++ (oocActionBar game creature)
      oocBars = List.filterMap bar myCreatures
      ooc = if (List.length oocBars) > 0
            then [vbox <| [h4 [] [text "Out Of Combat"]] ++ oocBars]
            else []
      targetSel =
        case model.selectedAbility of
          Just (cid, abid) ->
            case T.listFind (\c -> c.id == cid) myCreatures of
              Just _ -> [targetSelector model game (M.ActCreature cid) abid]
              Nothing -> []
          Nothing -> []
      comUI =
        case game.current_combat of
          Just combat -> [h4 [] [text "Combat"], playerCombatArea model game combat myCreatures]
          Nothing -> [text "No Combat"]
      movable mapc =
        -- Don't allow click-to-move when ANY creatures are in combat.
        -- TODO: maybe allow click-to-move for out-of-combat creatures
        case game.current_combat of
          Just com ->
            if List.member (T.combatCreature com) myCreatures && mapc.creature == T.combatCreature com
            then Just (\_ -> M.GetCombatMovementOptions)
            else Nothing
          Nothing -> Just M.GetMovementOptions
      currentCreature = Maybe.map (\com -> (T.combatCreature com).id) game.current_combat
      modifyMapCreature mapc =
        let highlight = (Just mapc.creature.id) == currentCreature
        in { mapc | movable = (movable mapc), highlight = highlight}
      vCreatures = List.map modifyMapCreature (visibleCreatures model game)
  in
    hbox <| [Grid.terrainMap model model.currentMap vCreatures
            , vabox [s [S.width (S.px 350)]] (comUI ++ ooc ++ targetSel)]

mapSelector : T.Game -> Html M.Msg
mapSelector game = vbox <|
  let mapSelectorItem name = button [onClick (M.SendCommand (T.SelectMap name))] [text name]
  in (List.map mapSelectorItem (Dict.keys game.maps))

inactiveList : M.Model -> T.Game -> Html M.Msg
inactiveList model game = div []
  [ div [] (List.map (inactiveEntry model game) (Dict.values game.creatures))
  ]


inactiveEntry : M.Model -> T.Game -> T.Creature -> Html M.Msg
inactiveEntry model game creature = vbox <|
  [hbox <|
    [ creatureCard [noteBox model creature] model creature
    ] ++ case game.current_combat of
        Just _ -> [engageButton creature]
        Nothing -> []
    ++ [
      deleteCreatureButton creature
    ], hbox (oocActionBar game creature)]

combatantList : Bool -> M.Model -> T.Game -> T.Combat -> Html M.Msg
combatantList isGmView model game combat =
  vbox (List.map (combatantEntry isGmView model game combat) (List.indexedMap (,) combat.creatures.data))

engageButton : T.Creature -> Html M.Msg
engageButton creature =
  button [onClick (M.SendCommand (T.AddCreatureToCombat creature.id))] [text "Engage"]

combatantEntry : Bool -> M.Model -> T.Game -> T.Combat -> (Int, T.Creature) -> Html M.Msg
combatantEntry isGmView model game combat (idx, creature) = hbox <|
  let marker = if combat.creatures.cursor == idx
               then [ datext [s [S.width (S.px 25)]] "▶️️" ]
               else []
      initiativeArrows =
        if isGmView
        then [ button [ onClick (M.SendCommand (T.ChangeCreatureInitiative creature.id (idx - 1)))
                      , disabled (idx == 0)]
                      [text "⬆️️"]
             , button [ onClick (M.SendCommand (T.ChangeCreatureInitiative creature.id (idx + 1)))
                      , disabled (idx == (List.length combat.creatures.data) - 1)]
                      [text "⬇️️"]
             ]
        else []
      gutter = [vabox [s [(S.width (S.px 25))]] <| marker ++ initiativeArrows]
  in gutter ++ [ creatureCard [] model creature ]

noteBox : M.Model -> T.Creature -> Html M.Msg
noteBox model creature = 
  let note = Maybe.withDefault creature.note (Dict.get creature.id model.creatureNotes)
      inp = input [type_ "text", value note, onInput (M.SetCreatureNote creature.id)] []
      saveButton =
        if creature.note /= note
        then [button [onClick (M.SendCommand (T.SetCreatureNote creature.id note))] [text "Save Note"]]
        else []
  in hbox <| [inp] ++ saveButton

disengageButton : T.Creature -> Html M.Msg
disengageButton creature =
  button [onClick (M.SendCommand (T.RemoveCreatureFromCombat creature.id))] [text ("Disengage " ++ creature.id)]

deleteCreatureButton : T.Creature -> Html M.Msg
deleteCreatureButton creature =
  button [onClick (M.SendCommand (T.RemoveCreature creature.id))] [text "Delete"]

