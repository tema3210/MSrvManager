import styled from "styled-components";

type Props = {
    onClick?: () => void
} & React.PropsWithChildren;

const Inner = styled.button`
    padding: 0.5rem;
    border: 3px solid transparent;
    border-left: 3px solid #db9f30;
    background-color: #ffffff5c;
    color: #db9f30;
    font-size: 1rem;
    margin: 0.5rem;
    cursor: pointer;

    &:hover {
        background-color: #ffffffbc;
    }

    &:active {
        background-color: #ffffffdc;
    }
`;

const Btn = ({onClick,children}: Props) => {
    return <Inner onClick={onClick ?? (()=>{}) }>{children}</Inner>
};

export default Btn;