function isPowerOf2(value) {
    return (value & (value - 1)) === 0;
}

export { isPowerOf2 };