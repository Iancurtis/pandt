import * as React from "react";
import * as ReactDOM from "react-dom";
import * as LD from "lodash";
import * as T from './PTTypes';

export function renderPlayerUI(
  elmApp: any,
  [id, playerID, currentScene, data]: [string, T.PlayerID, T.SceneID | undefined, any]) {
  let element = document.getElementById(id);
  console.log("[renderPlayerUI] Rendering Player component from Elm", id, element, playerID, currentScene);
  let app = T.decodeApp.decodeAny(data);
  ReactDOM.render(
    <PlayerUI app={app} playerID={playerID} currentScene={currentScene} />,
    element
  );
}

class PlayerUI extends React.Component<
  { playerID: T.PlayerID; currentScene: string | undefined; app: T.App; },
  undefined> {

  render(): JSX.Element {
    console.log("[PlayerUI:render]");
    return <div>
      <div>Player: {this.props.playerID}</div>
      <PlayerCreatures playerID={this.props.playerID} app={this.props.app} />
    </div>;
  }
}

class PlayerCreatures extends React.Component<{ playerID: T.PlayerID; app: T.App; }, undefined> {

  creatureSection(creature: T.Creature): JSX.Element {
    return <div key={creature.id}>
      <CreatureCard app={this.props.app} creature={creature} />
      <Collapsible name="Inventory">
        <CreatureInventory app={this.props.app} creature={creature} />
      </Collapsible>
    </div>;
  }

  render(): JSX.Element {
    let cids = this.props.app.players[this.props.playerID].creatures;
    let creatures = T.getCreatures(this.props.app, cids);
    return <div>
      {creatures.map(this.creatureSection.bind(this))}
    </div>
  }
}

class Collapsible extends React.Component<{ name: string }, { collapsed: boolean }> {
  constructor(props: { name: string }) {
    super(props);
    this.state = { collapsed: false };
  }
  toggle() {
    this.setState({ collapsed: !this.state.collapsed });
  }
  render(): JSX.Element {
    let buttonText, noneOrBlock;
    if (this.state.collapsed) {
      buttonText = "▶"; noneOrBlock = "none";
    }
    else {
      buttonText = "▼"; noneOrBlock = "block";
    };
    return <div>
      <div style={{ display: "flex" }}>
        <strong>{this.props.name}</strong>
        <button onClick={this.toggle.bind(this)}>{buttonText}</button>
      </div>
      <div style={{ display: noneOrBlock }}>{this.props.children}</div>
    </div>
  }
}


class CreatureCard extends React.Component<{ creature: T.Creature; app: T.App }, undefined> {
  render(): JSX.Element {
    let creature = this.props.creature;
    return <div
      style={{
        width: "300px",
        borderRadius: "10px", border: "1px solid black",
        padding: "3px"
      }}>
      <div>{classIcon(creature)} <strong>{creature.name}</strong>
        {LD.values(creature.conditions).map((ac) => conditionIcon(ac.condition))}
      </div>
      <CreatureIcon app={this.props.app} creature={creature} />
    </div>;
  }
}

function classIcon(creature: T.Creature): string {
  switch (creature.class_) {
    case "cleric": return "💉";
    case "rogue": return "🗡️";
    case "ranger": return "🏹";
    case "creature": return "🏃";
    case "baddie": return "👹";
    default: return ""
  }
}

function CreatureIcon(props: { app: T.App, creature: T.Creature }): JSX.Element | null {
  let squareStyle = { width: "50px", height: "50px", borderRadius: "10px", border: "solid 1px black" };
  if (props.creature.portrait_url !== "") {
    return <img src={props.creature.portrait_url}
      style={squareStyle} />
  } else {
    let class_ = props.app.current_game.classes[props.creature.class_];
    let color;
    if (class_) {
      color = class_.color;
    } else {
      color = "red";
    }
    return <div style={{ backgroundColor: color, ...squareStyle }}>{props.creature.name}</div>
  }
}

function CreatureInventory(props: { app: T.App, creature: T.Creature }): JSX.Element | null {
  let inv = props.creature.inventory;
  let items = T.getItems(props.app, LD.keys(inv));
  return <div>{items.map((item) => <div key={item.id}>{item.name} ({inv[item.id]})</div>
  )}</div>;
}

function conditionIcon(cond: T.Condition): string {
  switch (cond.t) {
    case "RecurringEffect": return cond.effect.toString();
    case "Dead": return "💀";
    case "Incapacitated": return "😞";
    case "AddDamageBuff": return "😈";
    case "DoubleMaxMovement": return "🏃";
    case "ActivateAbility": return "Ability Activated: " + cond.ability_id;
  }
}
