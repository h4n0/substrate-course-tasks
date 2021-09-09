import React, { useEffect, useState } from "react";
import { Form, Grid } from "semantic-ui-react";

import { useSubstrate } from "./substrate-lib";
import { TxButton } from "./substrate-lib/components";

import KittyCards from "./KittyCards";

export default function Kitties(props) {
  const { api, keyring } = useSubstrate();
  const { accountPair } = props;

  const [kitties, setKitties] = useState([]);
  const [status, setStatus] = useState("");
  const [kittiesCount, setKittiesCount] = useState(0);

  const fetchKitties = () => {
    api.query.kittiesModule.kittiesCount((c) => {
      let count = 0

      console.log("api changed")

      if (c.isNone) {
        console.log("c is none!!!")
        return
      } else {
        count = c.unwrap().toNumber()
        console.log("c is %d", count)
        setKittiesCount(count)
      }
    });
  };

  const populateKitties = () => {
    console.log("api changed pop");
    console.log("count is %d", kittiesCount)

    const kittiesRaw = [];
    for (let i = 0; i < kittiesCount; i++) {
      api.queryMulti(
        [
          [api.query.kittiesModule.kitties, i],
          [api.query.kittiesModule.owner, i],
        ],
        ([kittyRaw, owner]) => {
          const kitty = {
            id: i,
            dna: kittyRaw.unwrapOr(null).toU8a(),
            owner: owner.unwrapOr(null).toHuman(),
          }

          console.log(kitty);
          if (i >= kittiesRaw.length) {
            kittiesRaw.push(kitty)
          } else {
            kittiesRaw[i] = kitty
          }
          if (kittiesCount === kittiesRaw.length) {
            setKitties(kittiesRaw)
          }
        }
      );
    }
  };

  useEffect(fetchKitties, [api, keyring])
  useEffect(populateKitties, [api, kittiesCount])

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
              palletRpc: "kittiesModule",
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
