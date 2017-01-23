--- HACK'n'PASTED from elm-mouse-events
-- from "elm-dom"

module MouseEvent exposing (..)

import DOM exposing (Rectangle)
import Html
import Html.Events exposing (on)
import Json.Decode as Decode

type alias Position =
    { x : Int, y : Int }


type alias MouseEvent =
    { elementPos : Position
    , targetPos : Position
    }

mouseEvent : Int -> Int -> Rectangle -> MouseEvent
mouseEvent offsetX offsetY target =
    { elementPos = Position offsetX offsetY
    , targetPos = Position (truncate target.left) (truncate target.top)
    }

mouseEventDecoder : Decode.Decoder MouseEvent
mouseEventDecoder =
    Decode.map3
        mouseEvent
        (Decode.field "offsetX" Decode.int)
        (Decode.field "offsetY" Decode.int)
        (Decode.field "target" DOM.boundingClientRect)

onMouseClick : (MouseEvent -> msg) -> Html.Attribute msg
onMouseClick target = on "click" (Decode.map target mouseEventDecoder)
