import styled from "styled-components";
import { makeOnLoad, SSRProps } from "./lib";

interface ErrorMessageProps {
    color?: string;
    fontSize?: string;
    fontStyle?: string;
    title?: string;
}

const ErrorMessageContainer = styled.div`
    border: 1px solid ${({ color }) => color || '#e6551c'};
    padding: 1rem;
    border-radius: 5px;
    background-color: #fff5f5;
`;

const ErrorMessageTitle = styled.h2<ErrorMessageProps>`
    color: ${({ color }) => color || '#e6551c'};
    margin: 0;
    font-size: 1.5rem;
`;

const ErrorMessageText = styled.div<ErrorMessageProps>`
    color: ${({ color }) => color || '#e6551c'};
    font-size: ${({ fontSize }) => fontSize || '1.2rem'};
    font-style: ${({ fontStyle }) => fontStyle || 'italic'};
`;

const ErrorDisplay = ({pageData}: SSRProps) => {
    const { msg, color, fontSize, fontStyle, title } = pageData

    return (
        <ErrorMessageContainer color={color}>
            {title && <ErrorMessageTitle color={color}>{title}</ErrorMessageTitle>}
            <ErrorMessageText color={color} fontSize={fontSize} fontStyle={fontStyle}>{msg}</ErrorMessageText>
        </ErrorMessageContainer>
    );
};

window.onload = makeOnLoad(ErrorDisplay);