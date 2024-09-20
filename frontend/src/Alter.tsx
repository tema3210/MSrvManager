import { makeOnLoad, SSRProps } from "./lib";

const Alter = ({pageData}: SSRProps) => {

    return <p>ALTER {pageData.name}</p>
}

window.onload = makeOnLoad(Alter)