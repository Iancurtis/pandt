import * as LD from "lodash";
import * as React from "react";
import * as ReactDOM from "react-dom";

import * as CommonView from "./CommonView";
import * as Grid from './Grid';
import { PTUI } from './Model';
import * as M from './Model';
import * as T from './PTTypes';

export function renderPlayerUI(
  elmApp: any,
  [id, player_id, current_scene, data]: [string, T.PlayerID, T.SceneID | undefined, any]) {
  const element = document.getElementById(id);
  console.log("[renderPlayerUI] Rendering Player component from Elm", id, element, player_id, current_scene);
  const app = T.decodeApp.decodeAny(data);
  const ptui = new M.PTUI(elmApp, app);
  const player = M.get(ptui.app.players, player_id);
  if (player) {
    ReactDOM.render(
      <PlayerSideBar ptui={ptui} player={player} current_scene={current_scene} />,
      element
    );
  }
}

interface PlayerMainProps { app?: object; elm_app: any; }
export class PlayerMain extends React.Component<PlayerMainProps,
  { player_id: T.PlayerID | undefined; typing_player_id: string; ptui: M.PTUI | undefined }> {
  constructor(props: PlayerMainProps) {
    super(props);
    const ptui = props.app ? new M.PTUI(props.elm_app, T.decodeApp.decodeAny(props.app)) : undefined;
    this.state = { player_id: undefined, typing_player_id: "", ptui };
  }
  componentWillReceiveProps(nextProps: PlayerMainProps) {
    console.log("[PlayerMain:componentWillReceiveProps]");
    if (!LD.isEqual(this.props, nextProps)) {
      const ptui = nextProps.app ? new M.PTUI(nextProps.elm_app, T.decodeApp.decodeAny(nextProps.app))
        : undefined;
      this.setState({ ptui });
    }
  }

  render(): JSX.Element {
    if (!this.state.ptui) {
      return <div>Waiting for initial data from server.</div>;
    }
    if (this.state.player_id) {
      const player = M.get(this.state.ptui.app.players, this.state.player_id);
      if (player) {
        return <PlayerGameView player={player} ptui={this.state.ptui} />;
      } else {
        return <div>Couldn't find player {this.state.player_id}</div>;
      }
    } else {
      return <div>
        <h1>P&T</h1>
        <p>Welcome to P&T!</p>
        {LD.keys(this.state.ptui.app.players).length > 0
          ? <div>
            <p>You can rejoin a session if you've already registered as a player.</p>
            {LD.keys(this.state.ptui.app.players).map(pid =>
              <button key={pid} onClick={() => this.setState({ player_id: pid })}>{pid}</button>)}
          </div>
          : <noscript />}
        <p>You can register a new player. Enter your name (not your character's name) here:</p>
        <input type="text" value={this.state.typing_player_id}
          onChange={e => this.setState({ typing_player_id: e.currentTarget.value })} />
        <button
          onClick={() => this.registerPlayer()}>
          Register</button>
      </div>;
    }
  }
  registerPlayer() {
    if (!this.state.ptui) {
      console.log("[PlayerMain] registerPlayer called without PTUI state.");
      return;
    }
    this.state.ptui.sendCommand({ t: "RegisterPlayer", player_id: this.state.typing_player_id });
    this.setState({ player_id: this.state.typing_player_id });
  }
}

function PlayerGameView(props: { player: T.Player; ptui: M.PTUI }): JSX.Element {
  const grid = props.player.scene
    ? <Grid.Grid ptui={props.ptui} scene_id={props.player.scene} />
    : <div>No scene loaded</div>;

  return <div style={{ display: "flex", justifyContent: "space-between", height: "100%", width: "100%" }}>
    {grid}
    <div style={{ width: 450, height: "100%", border: "1px solid black" }}>
      <PlayerSideBar player={props.player} current_scene={props.player.scene} ptui={props.ptui} />
    </div>
  </div>;
}

function PlayerSideBar(props: { player: T.Player; current_scene: string | undefined; ptui: M.PTUI; })
  : JSX.Element {
  return <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
    <div style={{ flex: "1 0 auto" }}>
      <CommonView.TabbedView>
        <CommonView.Tab name="Creatures">
          <PlayerCreatures player={props.player} current_scene={props.current_scene} ptui={props.ptui} />
        </CommonView.Tab>
        <CommonView.Tab name="Combat">
          <CommonView.Combat ptui={props.ptui} />
        </CommonView.Tab>
        <CommonView.Tab name="Notes">
          <PlayerNote player_id={props.player.player_id} ptui={props.ptui} />
        </CommonView.Tab>
      </CommonView.TabbedView>
    </div>
    <div>
      <PlayerActionBar player={props.player} ptui={props.ptui} />
    </div>
  </div>;
}

function PlayerCreatures(
  props: { current_scene: T.SceneID | undefined; player: T.Player; ptui: M.PTUI; })
  : JSX.Element {
  const cids = props.player.creatures;
  const creatures = props.ptui.getCreatures(cids);
  console.log("[PlayerCreatures]", cids, creatures);
  if (creatures.length === 0) {
    return <div>You have no creatures in your control yet.</div>;
  }
  return <div>
    {creatures.map(creature =>
      <div key={creature.id}>
        <CommonView.CreatureCard app={props.ptui.app} creature={creature} />
        <div style={{ marginLeft: "1em" }}>
          <CommonView.Collapsible name="Inventory">
            <CommonView.CreatureInventory ptui={props.ptui} current_scene={props.current_scene}
              creature={creature} />
          </CommonView.Collapsible>
        </div>
      </div>
    )}
  </div>;
}

interface PlayerNoteProps { player_id: T.PlayerID; ptui: M.PTUI; }
class PlayerNote extends React.Component<PlayerNoteProps, { content: string | undefined }> {
  private path: Array<string>;
  constructor(props: PlayerNoteProps) {
    super(props);
    this.state = { content: undefined };
    this.path = ["Players", props.player_id];
  }

  render(): JSX.Element {
    const player_folder = this.props.ptui.getFolderNode(this.path);
    if (!player_folder) {
      return <div>Please ask your GM to creature the folder "{M.folderPathToString(this.path)}"</div>;
    }
    const note = M.get(player_folder.notes, "Scratch");
    const origContent = note ? note.content : "Enter notes here!";
    return <div>
      <div><button
        disabled={this.state.content === undefined || this.state.content === origContent}
        onClick={() => this.submit(note)}>Save</button></div>
      <div><textarea style={{ width: "100%", height: "100%" }}
        defaultValue={origContent} value={this.state.content}
        onChange={e => this.setState({ content: e.currentTarget.value })} /></div>
    </div>;
  }

  submit(origNote: T.Note | undefined) {
    if (!this.state.content) { return; }
    const note = { name: "Scratch", content: this.state.content };
    const cmd: T.GameCommand = origNote
      ? { t: "EditNote", path: this.path, name: "Scratch", note }
      : { t: "CreateNote", path: this.path, note };
    this.props.ptui.sendCommand(cmd);
  }
}

function PlayerActionBar(props: { player: T.Player; ptui: M.PTUI }): JSX.Element {
  if (props.ptui.app.current_game.current_combat) {
    const combat = props.ptui.app.current_game.current_combat;
    const cid = props.ptui.getCurrentCombatCreatureID(combat);
    if (LD.includes(props.player.creatures, cid)) {
      const creature = props.ptui.getCreature(cid);
      if (creature) {
        return <CommonView.ActionBar combat={combat} creature={creature} ptui={props.ptui} />;
      }
    }
  }
  return <noscript />;
}
