<<<<<<< HEAD
import Block from "./components/Block";

import { blockData } from "../stories/1-Block.stories";

export default function App() {
  return (
    <div>
      <div>Here is a simple block information</div>
      <Block {...blockData}></Block>
    </div>
  );
}
=======
export { default as Block } from "./components/Block";
export { default as BlockDetails } from "./components/BlockDetails";
export { default as Card } from "./components/Card";
export { default as CardStack } from "./components/CardStack";
export { default as ContentLabel } from "./components/ContentLabel";
export { default as Grid } from "./components/Grid";
export { default as GridItem } from "./components/GridItem";
export { default as Label } from "./components/Label";
export { default as SearchBar } from "./components/SearchBar";
export { default as TabLink } from "./components/TabLink";
export { default as TabNavigation } from "./components/TabNavigation";
>>>>>>> develop
