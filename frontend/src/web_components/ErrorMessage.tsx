import styled from "styled-components";

const ErrorMessage = styled.div`
    color: #e6551c;
    font-size: 1.2rem;
    font-style: italic;
`;

const C = ({msg}: {msg: string}) => {
    return <ErrorMessage>{msg}</ErrorMessage>
};

export default C;