int main() {
    int n = 1000000;

    long long a = 0;
    long long b = 1;

    for (int i = 0; i <= n; i++) {
        long long next = a + b;
        
        a = b;
        b = next;
    }

    return 0;
}