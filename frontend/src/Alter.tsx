import { makeOnLoad } from "./lib";

const Alter = () => {
    let pageProps = window.pageData;

    return <p>ALTER {pageProps.name}</p>
}

window.onload = makeOnLoad(<Alter />)