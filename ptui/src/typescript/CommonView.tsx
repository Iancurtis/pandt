import * as React from "react";
import * as ReactDOM from "react-dom";
import * as LD from "lodash";
import * as T from './PTTypes';

export class Collapsible extends React.Component<{ name: string }, { collapsed: boolean }> {
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

export class CreatureCard extends React.Component<{ creature: T.Creature; app: T.App }, undefined> {
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

export function classIcon(creature: T.Creature): string {
  switch (creature.class_) {
    case "cleric": return "💉";
    case "rogue": return "🗡️";
    case "ranger": return "🏹";
    case "creature": return "🏃";
    case "baddie": return "👹";
    default: return ""
  }
}

export function CreatureIcon(props: { app: T.App, creature: T.Creature }): JSX.Element | null {
  let squareStyle = { width: "50px", height: "50px", borderRadius: "10px", border: "solid 1px black" };
  if (props.creature.portrait_url !== "") {
    return <img src={props.creature.portrait_url}
      style={squareStyle} />
  } else {
    let class_ = props.app.current_game.classes[props.creature.class_];
    let color = class_ ? class_.color : "red";
    return <div style={{ backgroundColor: color, ...squareStyle }}>{props.creature.name}</div>
  }
}

interface CreatureInventoryProps { app: T.App; current_scene: T.SceneID | undefined; creature: T.Creature }
export class CreatureInventory extends React.Component<CreatureInventoryProps, { giving: T.ItemID | undefined }> {
  constructor(props: CreatureInventoryProps) {
    super(props);
    this.state = { giving: undefined };
  }
  render(): JSX.Element | null {
    let inv = this.props.creature.inventory;
    let items = T.getItems(this.props.app, LD.keys(inv));

    let give = this.state.giving
      ? <GiveItem app={this.props.app} current_scene={this.props.current_scene} giver={this.props.creature.id} item_id={this.state.giving} />
      : <noscript />;

    return <div>
      {items.map((item) =>
        <div key={item.id} style={{ display: "flex", justifyContent: "space-between" }}>
          {item.name} ({inv[item.id]})
        <button onClick={this.give.bind(this, item)}>Give</button>
        </div>
      )}
      {give}
    </div>;
  }

  give(item: T.Item) {
    this.setState({ giving: item.id });
  }
}

interface GiveItemProps { app: T.App; current_scene: T.SceneID | undefined; item_id: T.ItemID, giver: T.CreatureID }
export class GiveItem extends React.Component<GiveItemProps, { receiver: T.CreatureID | undefined; count: number | undefined }> {
  constructor(props: GiveItemProps) {
    super(props);
    this.state = { receiver: undefined, count: 1 };
  }
  render(): JSX.Element {
    if (!this.props.current_scene) { return <div>You can only transfer items in a scene.</div> }
    let scene = this.props.app.current_game.scenes[this.props.current_scene];
    if (!scene) { return <div>Couldn't find your scene</div> }
    let other_cids_in_scene = LD.keys(scene.creatures);
    LD.pull(other_cids_in_scene, this.props.giver);
    let other_creatures = T.getCreatures(this.props.app, other_cids_in_scene);
    if (!other_creatures) { return <div>There is nobody in this scene to give items to.</div> }
    let item = T.getItem(this.props.app, this.props.item_id);
    if (!item) { return <div>The Item definition cannot be found.</div> }
    let creature = T.getCreature(this.props.app, this.props.giver);
    if (!creature) { return <div>Giver not found!</div> }
    let giver_count = creature.inventory[this.props.item_id];
    if (!giver_count) { return <div>{creature.name} does not have any {item.name} to give.</div> }
    return <div>
      Giving
      <PositiveIntegerInput
        max={giver_count}
        onChange={(num) => this.setState({ count: num })}
        value={this.state.count} />
      {item.name}
      from {creature.name} to
      <select value={this.state.receiver} onChange={(ev) => this.onSelectCreature(ev)}>
        <option key="undefined" value="">Choose a creature</option>
        {other_creatures.map(
          (creature) => <option key={creature.id} value={creature.id}>{creature.name}</option>
        )}
      </select>
      <button disabled={!(this.state.receiver && this.state.count)}>Give</button>
      <button>Cancel</button>
    </div>;
  }

  onSelectCreature(event: React.SyntheticEvent<HTMLSelectElement>) {
    this.setState({ receiver: event.currentTarget.value });
  }

}

interface PositiveIntegerInputProps { max?: number; value: number | undefined; onChange: (num: number | undefined) => void }
export class PositiveIntegerInput extends React.Component<PositiveIntegerInputProps, undefined> {
  render(): JSX.Element {
    return <input type="text" value={this.props.value === undefined ? "" : this.props.value} onChange={(event) => {
      let num = Number(event.currentTarget.value);
      if (event.currentTarget.value === "") {
        this.props.onChange(undefined);
      } else if (num) {
        if (this.props.max !== undefined && num > this.props.max) { num = this.props.max; }
        this.props.onChange(num);
      }
    }} />
  }
}

export function conditionIcon(cond: T.Condition): string {
  switch (cond.t) {
    case "RecurringEffect": return cond.effect.toString();
    case "Dead": return "💀";
    case "Incapacitated": return "😞";
    case "AddDamageBuff": return "😈";
    case "DoubleMaxMovement": return "🏃";
    case "ActivateAbility": return "Ability Activated: " + cond.ability_id;
  }
}
