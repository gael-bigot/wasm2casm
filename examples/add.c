int add(int x, int y){
    return x + y;
}

int mul(int x, int y){
    return x * y;
}

int main(int x, int y, int z){
    return mul(x, add(y, z));
}
