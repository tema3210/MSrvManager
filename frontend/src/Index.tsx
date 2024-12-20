import { useState } from "react";
import { makeOnLoad, SSRProps } from "./lib";
import { InstanceDescriptor } from "./model";
import InstanceDisplay from "./components/InstanceDesc";
import InstanceActions from "./components/InstanceActions";
import Btn from "./components/Button";

import { gql, useSubscription, useQuery } from "@apollo/client";
import styled from "styled-components";
import { BrokenServerLink, InstanceStateDisplay, TextBig, TextSmall } from "./components/UIComps";
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

const ListWrapper = styled.div`
    height: 90vh;
    overflow-y: scroll;
`;

type ServerData = {
    servers: Record<string,{
        data: InstanceDescriptor,
        state: string
    }>
}

const Index = ({}: SSRProps) => {

    const { data: AVdata, loading: AVloading } = useQuery(gql`
        query {
            appVersion
        }
    `);

    const { data, loading, error } = useSubscription<ServerData>(gql`
        subscription {
            servers
        }
    `);

    const { data: brokens } = useSubscription<{ brokenServers: string[] }>(gql`
        subscription {
            brokenServers
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
            <TextBig>MSRVMANAGER: {(AVloading)? "-" : AVdata.appVersion }; We have these servers:</TextBig><br />
            <>
                {
                    (brokens)
                        ? <TextSmall>Broken servers: {brokens?.brokenServers.map((val) => (<BrokenServerLink href={`/renew?name=${val}`}>{val}</BrokenServerLink>))}</TextSmall>
                        : null
                }
            </>
            <ListWrapper>
                {
                    Object.entries((data?.servers ?? {}))
                        .sort(([a], [b]) => a.localeCompare(b))
                        .map(([name,{data,state}]) => {
                            switch (state) {
                                case "Stopped":
                                case "Running":
                                case "Crashed":
                                    return (
                                        <InstanceDisplay
                                            key={name}
                                            instance={data}
                                            state={state}
                                            selected={selected === name}
                                            setSelected={
                                                (selected === name)
                                                    ? () => setSelected(null)
                                                    : () => setSelected(name)
                                            }
                                        />
                                    );
                                default:
                                    return <InstanceStateDisplay serverName={name} state={state}/>
                            }1
                            
                        })
                }
            </ListWrapper>
        </InstanceWrapper>
        <InstanceWrapper width="25%">
            <TextBig>Actions:</TextBig><br />
            <Btn onClick={createOnClick}>Create Server =&gt;</Btn><br />
            {
                (selected && data?.servers?.[selected].data) ? <InstanceActions name={selected} state={data.servers[selected].state} deselect={() => setSelected(null)}/> : null
            }
        </InstanceWrapper>
    </Wrapper>
}

window.onload = makeOnLoad(Index)
