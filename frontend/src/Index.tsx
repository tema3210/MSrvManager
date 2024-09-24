import { useState } from "react";
import { makeOnLoad, SSRProps } from "./lib";
import { InstanceDescriptor } from "./model";
import InstanceDisplay from "./components/InstanceDesc";
import InstanceActions from "./components/InstanceActions";
import Btn from "./components/Button";

import { gql, useSubscription, useQuery } from "@apollo/client";
import styled from "styled-components";
import { TextBig } from "./components/UIComps";
import Spinner from "./components/Spinner";

const Wrapper = styled.div`
    display: flex;
    flex-direction: row;
    align-items: stretch;
`;

const InstanceWrapper = styled.div<{width: string}>`
    margin-left: 1rem;
    margin-right: 1rem;
    width: ${({width}) => `calc(${width} - 2rem)`};
`;

const Footer = styled.div`
    position: absolute;
    bottom: 0;
    height: 3rem;
`;

const Index = ({}: SSRProps) => {

    const { data: AVdata, loading: AVloading } = useQuery(gql`
        query {
            appVersion
        }
    `);

    const { data, loading, error } = useSubscription<{servers: Record<string,InstanceDescriptor>}>(gql`
        subscription {
            servers
        }
    `);

    const [selected,setSelected] = useState<string | null>(null);

    if (loading) return <Spinner />;
    if (error) return <pre>{error.message}</pre>

    const createOnClick = () => {
        window.location.href = '/create';
    };
    
    return <Wrapper>
        <InstanceWrapper width="75%">
            <TextBig>We have these servers:</TextBig>
            {
                Object.entries((data!.servers ?? {}))
                    .map(([name,desc]) => (
                        <InstanceDisplay
                            key={name}
                            instance={desc}
                            selected={selected === name}
                            setSelected={
                                (selected === name)
                                    ? () => setSelected(null)
                                    : () => setSelected(name)
                            }
                        />
                    ))
            }
        </InstanceWrapper>
        <InstanceWrapper width="25%">
            <TextBig>Actions:</TextBig><br />
            <Btn onClick={createOnClick}>Create Server =&gt;</Btn>
            {
                (selected) ? <InstanceActions instance={(data!.servers ?? {})[selected]} deselect={() => setSelected(null)}/> : null
            }
        </InstanceWrapper>
        
        <Footer><TextBig>Version: {(AVloading)? "-" : AVdata.appVersion }</TextBig></Footer>
    </Wrapper>
}

window.onload = makeOnLoad(Index)
