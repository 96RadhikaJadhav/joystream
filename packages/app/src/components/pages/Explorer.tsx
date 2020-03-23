import React, { useState } from "react";
import { TabNavigation, SearchBar, CardStack, Block } from "components";

let links = [
  {
    label: "Chain Info",
    to: "#",
  },
  {
    label: "Block Details",
    to: "#",
  },
  {
    label: "Forks",
    to: "#",
    isSelected: true,
  },
  {
    label: "Node Info",
    to: "#",
  },
];

const blockData = {
  blockNum: 1854,
  hash: "0x080d672d268a72cae5b255918c1c832439869d6b9b36933734612ba7cd53f2db",

  parentHash:
    "0xdfe6522b146213accfb65c16ba82ad96e0cc737abe21eb28bbea80135e08e2d4",

  stateRoot:
    "0xb0486392387dc820afb96ecd3b0d8f129538b789d811d0e82d86607d45424664",

  extrinsictRoot:
    "0x3f4378235b086fd65dd5d3650c8ef82e1b75297e884389ad1037fcc966a9ae05",
};

let recentBlocks = [
  <Block {...blockData} isExpanded={true} />,
  <Block {...blockData} />,
  <Block {...blockData} />,
  <Block {...blockData} />,
];

let recentEvents = Array.from({ length: 5 }, (_, i) => (
  <span>{`Card number: ${i}`}</span>
));

export default function Explorer() {
  let [tab, setTab] = useState(0);
  return (
    <>
      <header>
        <TabNavigation selected={tab} links={links} />
        <SearchBar placeholder={"block hash or number to query..."} />
      </header>
      <main>
        <div>
          <CardStack items={recentBlocks} />
          <CardStack items={recentEvents} />
        </div>
      </main>
    </>
  );
}
