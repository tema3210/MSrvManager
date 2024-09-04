
import r2wc from "@r2wc/react-to-web-component"

import Hello from './src/Hello.tsx';
type HelloProps = Parameters<typeof Hello>[0];
const HelloElement = r2wc<HelloProps>(Hello,{props: {name: "string",age: "number",flag: "boolean",array: "json",}});
customElements.define('c-hello', HelloElement);

declare module "react" {
  namespace JSX {
    interface IntrinsicElements {
      "c-hello": JSX.IntrinsicAttributes & HelloProps;
    }
  }
}

import Test from './src/Test.tsx';
type TestProps = Parameters<typeof Test>[0];
const TestElement = r2wc<TestProps>(Test,{props: {f: "function",obj: "json",name: "string",}});
customElements.define('c-test', TestElement);

declare module "react" {
  namespace JSX {
    interface IntrinsicElements {
      "c-test": JSX.IntrinsicAttributes & TestProps;
    }
  }
}
