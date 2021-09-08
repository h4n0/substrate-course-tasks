import React, { useEffect, useState } from "react";
import { Form, Grid } from "semantic-ui-react";

import { useSubstrate } from "./substrate-lib"
import { TxButton } from "./substrate-lib/components";

import KittyCards from "./KittyCards"

export default function Kitties(props) {
  const { api, keyring } = useSubstrate()
  const { accountPair } = props

  const [kitties, setKitties] = useState([]);
  const [status, setStatus] = useState('');

  const [newKitty, setNewKitty] = useState(null);
  const [kittiesCount, setKittiesCount] = useState(0);
  const [kittiesDna, setKittiesDna] = useState([]);
  const [kittiesOwner, setKittiesOwner] = useState([]);

  const fetchKitties = () => {
    // TODO: 在这里调用 `api.query.kittiesModule.*` 函数去取得猫咪的信息。
    // 你需要取得：
    //   - 共有多少只猫咪
    //   - 每只猫咪的主人是谁
    //   - 每只猫咪的 DNA 是什么，用来组合出它的形态
    api.query.kitties.kittiesCount((c) => {
      let count = 0;

      if (c.isNone) {
        return;
      } else {
        count = c.unwrap().toNumber();
      }

      let startIndex = 0;
      if (kitties.length < count - 1) {
        // Previous kitties missing, add them
        startIndex = 1;
      } else if (kitties.length > count - 1) {
        // Rarely happen, maybe blockchain restarted, then reset kitties
        setKitties([]);
        startIndex = 1;
      } else {
		  // The newly added kitty is still not in kitties, therefore add this only
		  startIndex = count;
	  }

      for (let i = startIndex; i <= count; i++) {
        api.queryMulti(
          [
            [api.query.kitties.kitties, i],
            [api.query.kitties.owner, i],
          ],
          ([kittyRaw, owner]) => {
            const kitty = {
              id: i,
              dna: kittyRaw.unwrapOr(null).toU8a(),
              owner: owner.unwrapOr(null).toHuman(),
            };

            setNewKitty(kitty);
            console.log(kitty);
          }
        );
      }
    });
  };

  const populateKitties = () => {
    if (newKitty && newKitty.id > kitties.length) {
      kitties.push(newKitty);
      setKitties(kitties);
            console.log(kitties);
    }
  };

  useEffect(fetchKitties, [api, keyring]);
  useEffect(populateKitties, [newKitty]);

  return (
    <Grid.Column width={16}>
      <h1>小毛孩</h1>
      <KittyCards
        kitties={kitties}
        accountPair={accountPair}
        setStatus={setStatus}
      />
      <Form style={{ margin: "1em 0" }}>
        <Form.Field style={{ textAlign: "center" }}>
          <TxButton
            accountPair={accountPair}
            label="创建小毛孩"
            type="SIGNED-TX"
            setStatus={setStatus}
            attrs={{
              palletRpc: "kitties",
              callable: "create",
              inputParams: [],
              paramFields: [],
            }}
          />
        </Form.Field>
      </Form>
      <div style={{ overflowWrap: "break-word" }}>{status}</div>
    </Grid.Column>
  );
}
