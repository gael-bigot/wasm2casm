int add(int x, int y){
    return x + y;
}

int mul(int x, int y){
    return x * y;
}

int foo(int a, int b, int c){
    int d = 3 * add(b, c);
    return mul(a, d) - 12;
}
