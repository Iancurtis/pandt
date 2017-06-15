import * as LD from "lodash";
import * as React from "react";
import * as ReactDOM from "react-dom";
import * as ReactRedux from "react-redux";
import * as Redux from "redux";
import * as svgPanZoom from "svg-pan-zoom";

import * as CommonView from "./CommonView";
import { PTUI } from "./Model";
import * as M from "./Model";
import * as T from "./PTTypes";

interface Obj<T> { [index: string]: T; }

export interface GridProps {
  scene: T.Scene;
  creatures: Obj<MapCreature>;
}

export function grid_comp(props: GridProps & M.ReduxProps) {
  const map_ = M.get(props.ptui.app.current_game.maps, props.scene.map);
  if (!map_) { return <div>Couldn't find map</div>; }
  const map = map_; // WHY TYPESCRIPT, WHY???

  const grid = props.ptui.state.grid;

  const menu = grid.active_menu ? renderMenu(grid.active_menu) : <noscript />;
  const annotation = grid.display_annotation ? renderAnnotation(grid.display_annotation)
    : <noscript />;

  return <div style={{ width: "100%", height: "100%" }}>
    {menu}
    {annotation}
    <GridSvg map={map} creatures={LD.values(props.creatures)} />
  </div>;

  function renderAnnotation({ pt, rect }: { pt: T.Point3, rect: M.Rect }): JSX.Element {
    const special = LD.find(map.specials, ([pt_, _, note, _2]) => M.isEqual(pt, pt_));
    if (!special) { return <noscript />; }
    return <div style={{
      position: "fixed",
      fontSize: "24px",
      top: rect.sw.y, left: rect.sw.x,
      border: "1px solid black", borderRadius: "5px",
      backgroundColor: "white",
    }}>{special[2]}</div>;
  }

  function renderMenu({ cid, rect }: { cid: T.CreatureID, rect: M.Rect }): JSX.Element {
    const creature_ = M.get(props.creatures, cid);
    if (!creature_) {
      return <noscript />;
    }
    const creature = creature_; // WHY TYPESCRIPT, WHY???
    return <div
      style={{
        position: "fixed",
        paddingLeft: "0.5em",
        paddingRight: "0.5em",
        top: rect.sw.y, left: rect.sw.x,
        backgroundColor: "white",
        border: "1px solid black",
        borderRadius: "5px",
      }}
    >
      <div style={{ borderBottom: "1px solid grey" }}>{creature.creature.name}</div>
      <div style={{ borderBottom: "1px solid grey" }}>
        <a style={{ cursor: "pointer" }}
          onClick={() => props.dispatch({ type: "ActivateGridCreature", cid, rect })}>
          (Close menu)</a>
      </div>
      {
        LD.keys(creature.actions).map(
          actionName => {
            function onClick() {
              props.dispatch({ type: "ActivateGridCreature", cid, rect });
              creature.actions[actionName](cid);
            }
            return <div key={actionName}
              style={{ fontSize: "24px", borderBottom: "1px solid grey", cursor: "pointer" }}
            >
              <a onClick={() => onClick()}>{actionName}</a>
            </div>;
          })
      }
    </div>;
  }
}

export const Grid = M.connectRedux(grid_comp);


export interface MapCreature {
  creature: T.Creature;
  pos: T.Point3;
  class_: T.Class;
  actions: { [index: string]: (cid: T.CreatureID) => void };
}

interface GridSvgProps {
  map: T.Map;
  creatures: Array<MapCreature>;
}
interface GridSvgState { spz_element: SvgPanZoom.Instance | undefined; }
class GridSvgComp extends React.Component<GridSvgProps & M.ReduxProps, GridSvgState> {

  constructor(props: GridSvgProps & M.ReduxProps) {
    super(props);
    this.state = { spz_element: undefined };
  }

  componentDidMount() {
    const pz = svgPanZoom("#pt-grid", {
      dblClickZoomEnabled: false,
      center: true,
      fit: true,
      // TODO: Hammer.js integration
      // , customEventsHandler: eventsHandler
      zoomScaleSensitivity: 0.5,
    });
    this.setState({ spz_element: pz });
  }

  componentWillUnmount() {
    if (this.state.spz_element) {
      this.state.spz_element.destroy();
    }
  }

  shouldComponentUpdate(nextProps: GridSvgProps & M.ReduxProps): boolean {
    const mvmt_diff = !M.isEqual(
      this.props.ptui.state.grid.movement_options,
      nextProps.ptui.state.grid.movement_options);
    const app_diff = !M.isEqual(this.props.ptui.app, nextProps.ptui.app);
    return app_diff || mvmt_diff;
  }

  render(): JSX.Element {
    const { map, creatures, ptui } = this.props;
    console.log("[EXPENSIVE:GridSvg.render]");
    const terrain_els = map.terrain.map(pt => tile("white", "base-terrain", pt));
    const creature_els = creatures.map(
      c => <GridCreature key={c.creature.id} creature={c} />);
    const move = ptui.state.grid.movement_options;
    const movement_target_els = move
      ? move.options.map(pt => <MovementTarget key={pt.toString()} cid={move.cid} pt={pt} />)
      : [];
    const special_els = this.props.map.specials.map(
      ([pt, color, _, vis]) => <SpecialTile key={pt.toString()} pt={pt} color={color} vis={vis} />);
    const annotation_els = M.filterMap(this.props.map.specials,
      ([pt, _, note, vis]) => {
        if (note !== "") {
          return <Annotation key={pt.toString()} pt={pt} note={note} vis={vis} />;
        }
      });

    return <svg id="pt-grid" preserveAspectRatio="xMinYMid slice"
      style={{ width: "100%", height: "100%", backgroundColor: "rgb(215, 215, 215)" }}>
      <g>
        {/* this <g> needs to be here for svg-pan-zoom. Otherwise svg-pan-zoom will reparent all
          nodes inside the <svg> tag to a <g> that it controls, which will mess up react's
          virtualdom rendering */}
        {terrain_els}
        {creature_els}
        {movement_target_els}
        {special_els}
        {annotation_els}
      </g>
    </svg>;
  }
}
export const GridSvg = M.connectRedux(GridSvgComp);

function movementTarget(
  { cid, pt, ptui, dispatch }: { cid: T.CreatureID; pt: T.Point3 } & M.ReduxProps)
  : JSX.Element {
  const tprops = tile_props("cyan", pt);
  return <rect {...tprops} fillOpacity="0.4"
    onClick={() => ptui.moveCreature(dispatch, cid, pt)} />;
}
const MovementTarget = M.connectRedux(movementTarget);

function specialTile(
  { color, vis, pt, ptui }: { color: string, vis: T.Visibility, pt: T.Point3 } & M.ReduxProps)
  : JSX.Element {
  if (M.isEqual(vis, { t: "GMOnly" }) && ptui.state.player_id) {
    return <noscript />;
  }
  const tprops = tile_props(color, pt);
  return <rect {...tprops} />;
}
const SpecialTile = M.connectRedux(specialTile);


function annotation(
  { ptui, dispatch, pt, note, vis }:
    { pt: T.Point3, note: string, vis: T.Visibility } & M.ReduxProps)
  : JSX.Element {
  if (M.isEqual(vis, { t: "GMOnly" }) && ptui.state.player_id) {
    return <noscript />;
  }

  let element: SVGRectElement;

  function onClick() {
    dispatch({ type: "ToggleAnnotation", pt, rect: screenCoordsForRect(element) });
  }

  return <g>
    <rect width="100" height="100" x={pt[0] * 100} y={pt[1] * 100 - 50}
      onClick={() => onClick()}
      fillOpacity="0"
      ref={el => element = el} />
    <text
      style={{ pointerEvents: "none" }}
      x={pt[0] * 100 + 25} y={pt[1] * 100 + 50}
      fontSize="100px" stroke="black" strokeWidth="2px" fill="white">*</text>
  </g>;
}
const Annotation = M.connectRedux(annotation);


interface GridCreatureProps { creature: MapCreature; }
function gridCreature_comp({ creature, dispatch }: GridCreatureProps & M.ReduxProps): JSX.Element {
  let element: SVGRectElement | SVGImageElement;
  function onClick() {
    const act: M.Action = {
      type: "ActivateGridCreature", cid: creature.creature.id, rect: screenCoordsForRect(element),
    };
    dispatch(act);
  }
  if (creature.creature.portrait_url !== "") {
    const props = tile_props("white", creature.pos, creature.creature.size);
    return <image ref={el => element = el} key={creature.creature.id} onClick={() => onClick()}
      xlinkHref={creature.creature.portrait_url} {...props} />;
  } else {
    const props = tile_props(creature.class_.color, creature.pos, creature.creature.size);
    return <g key={creature.creature.name} onClick={() => onClick()}>
      {<rect ref={el => element = el} {...props} />}
      {text_tile(creature.creature.name.slice(0, 4), creature.pos)}
    </g >;
  }
}

const GridCreature = M.connectRedux(gridCreature_comp);

function text_tile(text: string, pos: T.Point3): JSX.Element {
  return <text style={{ pointerEvents: "none" }} fontSize="50" x={pos[0] * 100} y={pos[1] * 100}>
    {text}
  </text>;
}

function tile(color: string, keyPrefix: string, pos: T.Point3, size?: { x: number, y: number })
  : JSX.Element {
  const key = `${keyPrefix}-${pos[0]}-${pos[1]}`;
  const props = tile_props(color, pos, size);
  return <rect key={key} {...props} />;
}

function tile_props(color: string, pt: T.Point3, size = { x: 1, y: 1 }): React.SVGProps<SVGElement> {
  return {
    width: 100 * size.x, height: 100 * size.y,
    rx: 5, ry: 5,
    x: pt[0] * 100, y: (pt[1] * 100) - 50,
    stroke: "black", strokeWidth: 1,
    fill: color,
  };
}

function screenCoordsForRect(rect: SVGRectElement | SVGImageElement): M.Rect {
  const svg = document.getElementById("pt-grid") as any as SVGSVGElement;
  const matrix = rect.getScreenCTM();
  const pt = svg.createSVGPoint();
  pt.x = rect.x.animVal.value;
  pt.y = rect.y.animVal.value;
  const nw = pt.matrixTransform(matrix);
  pt.x += rect.width.animVal.value;
  const ne = pt.matrixTransform(matrix);
  pt.y += rect.height.animVal.value;
  const se = pt.matrixTransform(matrix);
  pt.x -= rect.width.animVal.value;
  const sw = pt.matrixTransform(matrix);
  return { nw, ne, se, sw };
}
