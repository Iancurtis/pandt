import * as React from "react";
import * as ReactDOM from "react-dom";
import * as T from './PTTypes';
import * as CommonView from './CommonView';
import { PTUI } from './Model';
import * as M from './Model';
import * as LD from 'lodash';

import * as svgPanZoom from 'svg-pan-zoom';

interface GridProps {
  ptui: M.PTUI;
  scene_id: T.SceneID;
}

export class Grid extends React.Component<GridProps, { spz_element: object | undefined }> {
  constructor(props: GridProps) {
    super(props);
    this.state = { spz_element: undefined };
  }

  componentDidMount() {
    let pz = svgPanZoom("#pt-grid", {
      dblClickZoomEnabled: false,
      // , resize: true
      center: true,
      fit: true,
      // , customEventsHandler: eventsHandler
      zoomScaleSensitivity: 0.5,
    });
    this.setState({ spz_element: pz });
  }

  render(): JSX.Element {
    let scene = M.get(this.props.ptui.app.current_game.scenes, this.props.scene_id);
    if (!scene) { return <div>Couldn't find scene</div>; }
    let map = M.get(this.props.ptui.app.current_game.maps, scene.map);
    if (!map) { return <div>Couldn't find map</div>; }

    let open_terrain = map.terrain;
    let terrain_els = open_terrain.map((pt) => tile("white", "base-terrain", pt));

    return <svg id="pt-grid" preserveAspectRatio="xMinYMid slice"
      style={{ width: "100%", height: "100%", backgroundColor: "rgb(215, 215, 215)" }}>
      {terrain_els}
    </svg>;
  }
}

function tile(color: string, keyPrefix: string, [ptx, pty, _]: T.Point3): JSX.Element {
  let key = `${keyPrefix}-${ptx}-${pty}`;
  return <rect key={key} width={100} height={100} x={ptx * 100} y={pty * 100}
    fill={color} stroke="black" strokeWidth="1" />
}
