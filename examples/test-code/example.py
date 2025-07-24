def greet(name):
    return f"Hello, {name}!"

def calculate_factorial(n):
    if n <= 1:
        return 1
    return n * calculate_factorial(n - 1)

def main():
    print(greet("World"))
    print(f"Factorial of 5: {calculate_factorial(5)}")

if __name__ == "__main__":
    main()
