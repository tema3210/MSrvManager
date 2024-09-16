import styled from "styled-components";

const ErrorMessage = styled.div`
    color: #e6551c;
    font-size: 1.2rem;
    font-style: italic;
`;

export default ({msg}: {msg: string}) => {
    <ErrorMessage>{msg}</ErrorMessage>
};