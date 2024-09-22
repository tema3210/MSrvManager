import { Project } from "ts-morph";
import * as fs from "fs";
import * as path from "path";

function capitalize(str) {
  return str.charAt(0).toUpperCase() + str.slice(1);
}

const WEBCOMPONENTS_DIR = "./src/web_components"
const OUTPUT_FILE = "web_components.tsx"

const registerJSX = (el_name,ty) => {
  return `
declare module "react" {
  namespace JSX {
    interface IntrinsicElements {
      "${el_name}": JSX.IntrinsicAttributes & ${ty};
    }
  }
}`
}

// Generate import and registration code
let fileContent = `
import r2wc from "@r2wc/react-to-web-component"
`;

// Get all component files
const componentFiles = fs.readdirSync(WEBCOMPONENTS_DIR).filter(file => file.endsWith('.tsx'));

// Append import statements and component registration code
componentFiles.forEach(file => {
  const componentName = path.basename(file, path.extname(file));
  const importName = capitalize(componentName);
  const importPath = `./src/web_components/${file}`;

  let rwcOptions = ``;

  const isFunctionLike = (type) => {
    const callSigs = type.getCallSignatures();
  
    if (callSigs.length !== 1) {
      return false
    }
    return true
  }

  try {
    
    const project = new Project({
      tsConfigFilePath: "./tsconfig.json",
    });

    const [source] = project.addSourceFilesAtPaths([
      importPath
    ]);

    const symbol = source.getDefaultExportSymbol();

    if (!symbol) {
      return
    };

    const declarations = symbol.getDeclarations();

    if (declarations.length === 0) {
      console.error("No declarations found for the default export symbol.");
      return;
    }

    // Attempt to get the value declaration of the symbol
    const [symbolDeclaration] = symbol.getDeclarations().filter((d) => ['FunctionDeclaration','ExportAssignment'].includes(d.getKindName()));
    // we want FunctionDeclaration, ExportAssignment

    // Check if the value declaration exists
    if (!symbolDeclaration) {
      console.error("No value declaration found for the default export symbol.");
      return;
    }

    // Get the type of the default export symbol
    const type = symbol.getTypeAtLocation(symbolDeclaration);

    // If the type is not found, log an error and return
    if (!type) {
      console.error("Could not determine the type of the default export symbol.");
      return;
    }

    //the function component
    if (isFunctionLike(type)) {

      const callSigs = type.getCallSignatures();

      const [callSig] = callSigs;

      const params = callSig.getParameters();
      
      if (params.length != 1) {
        console.error("function is not a component")
        return 
      }

      const [props] = params;

      const propsType = props.getTypeAtLocation(symbolDeclaration);

      // thereafter we make props decl
      let propDesc = '';
      propsType.getProperties().forEach((prop) => {
        const propName = prop.getName();
        const propTy = prop.getTypeAtLocation(symbolDeclaration);

        // const isOptional = prop.isOptional() ? '?' : '';
      
        // console.log(`Processing property: ${propName}, Type: ${propTy.getText()}, Optional: ${isOptional}`);

        const isUnion = propTy.isUnion();
        const unionTypes = isUnion ? propTy.getUnionTypes() : [propTy];

        const isType = (typeChecked) => unionTypes.some(typeChecked);
      
        if (isType(type => type.isArray() || type.isObject())) {
          propDesc += `${propName}: "json",`;
          return;
        }
      
        if (isType(type => type.isBoolean())) {
          propDesc += `${propName}: "boolean",`;
          return;
        }
      
        if (isType(type => type.isString())) {
          propDesc += `${propName}: "string",`;
          return;
        }
      
        if (isType(type => type.isNumber() || type.isBigInt())) {
          propDesc += `${propName}: "number",`;
          return;
        }
      
        console.error(`${importPath}: ${propTy.getText()} cannot be passed through RWC`);
      });
      
      rwcOptions = `{props: {${propDesc}}}`;
    } else {
      return
    }

  } catch (e) {
    throw e;
  }

  fileContent += `
import ${importName} from '${importPath}';
type ${importName}Props = Parameters<typeof ${importName}>[0];
const ${importName}Element = r2wc<${importName}Props>(${importName},${rwcOptions});
customElements.define('r-${componentName.toLowerCase()}', ${importName}Element);
${registerJSX(`r-${componentName.toLowerCase()}`,`${importName}Props`)}
`;
});

// Write the generated content to the output file
fs.writeFileSync(OUTPUT_FILE, fileContent);


