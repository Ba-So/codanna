/**
 * @file comprehensive.cpp
 * @brief Comprehensive C++ example for parser testing - diverse language constructs
 * @author Code Intelligence System
 * 
 * This file demonstrates key C++ language features with minimal repetition:
 * - Classes, inheritance, virtual functions
 * - Templates and generic programming
 * - Namespaces and scope resolution
 * - Modern C++ features (auto, lambda, smart pointers)
 * - Exception handling and RAII
 * - Operator overloading and special member functions
 */

#include <iostream>
#include <vector>
#include <memory>
#include <string>
#include <algorithm>
#include <functional>
#include <exception>

/**
 * @namespace geometry
 * @brief Namespace containing geometric shapes and operations
 */
namespace geometry {
    
    /**
     * @class Shape
     * @brief Abstract base class for geometric shapes
     * 
     * Demonstrates virtual functions, pure virtual methods,
     * and polymorphic interfaces in C++.
     */
    class Shape {
    public:
        /**
         * @brief Virtual destructor for proper cleanup
         */
        virtual ~Shape() = default;
        
        /**
         * @brief Pure virtual function for area calculation
         * @return Area of the shape
         */
        virtual double area() const = 0;
        
        /**
         * @brief Virtual function for perimeter calculation
         * @return Perimeter of the shape
         */
        virtual double perimeter() const { return 0.0; }
        
        /**
         * @brief Non-virtual function with default implementation
         */
        void display() const {
            std::cout << "Shape with area: " << area() << std::endl;
        }
    };
    
    /**
     * @class Circle
     * @brief Concrete implementation of circular shape
     * 
     * Demonstrates inheritance, constructor initialization lists,
     * and const member functions.
     */
    class Circle : public Shape {
    private:
        double radius_;  ///< Circle radius
        
    public:
        /**
         * @brief Constructor with initialization list
         * @param r Circle radius
         */
        explicit Circle(double r) : radius_(r) {}
        
        /**
         * @brief Override area calculation for circle
         */
        double area() const override {
            return 3.14159 * radius_ * radius_;
        }
        
        /**
         * @brief Override perimeter calculation for circle
         */
        double perimeter() const override {
            return 2 * 3.14159 * radius_;
        }
        
        /**
         * @brief Getter for radius
         */
        double radius() const noexcept { return radius_; }
    };
    
    /**
     * @class Rectangle
     * @brief Rectangle shape with multiple inheritance demonstration
     */
    class Rectangle : public Shape {
    private:
        double width_, height_;
        
    public:
        Rectangle(double w, double h) : width_(w), height_(h) {}
        
        double area() const override {
            return width_ * height_;
        }
        
        double perimeter() const override {
            return 2 * (width_ + height_);
        }
    };
    
} // namespace geometry

/**
 * @namespace utils
 * @brief Utility namespace with template functions and algorithms
 */
namespace utils {
    
    /**
     * @brief Template function for generic comparison
     * @tparam T Type of elements to compare
     * @param a First element
     * @param b Second element
     * @return Maximum of the two elements
     */
    template<typename T>
    constexpr T max(const T& a, const T& b) {
        return (a > b) ? a : b;
    }
    
    /**
     * @brief Template class for generic container operations
     * @tparam Container Container type
     * @tparam Predicate Predicate function type
     */
    template<class Container, class Predicate>
    auto filter(const Container& container, Predicate pred) -> std::vector<typename Container::value_type> {
        std::vector<typename Container::value_type> result;
        std::copy_if(container.begin(), container.end(), 
                    std::back_inserter(result), pred);
        return result;
    }
    
    /**
     * @brief Template specialization example
     */
    template<>
    constexpr int max<int>(const int& a, const int& b) {
        return (a > b) ? a : b;
    }
    
} // namespace utils

/**
 * @class ResourceManager
 * @brief RAII resource management demonstration
 * 
 * Shows constructor/destructor patterns, move semantics,
 * and proper resource management in C++.
 */
class ResourceManager {
private:
    std::unique_ptr<int[]> data_;
    size_t size_;
    
public:
    /**
     * @brief Constructor with resource allocation
     */
    explicit ResourceManager(size_t size) 
        : data_(std::make_unique<int[]>(size)), size_(size) {
        std::cout << "ResourceManager allocated " << size << " integers\n";
    }
    
    /**
     * @brief Move constructor
     */
    ResourceManager(ResourceManager&& other) noexcept 
        : data_(std::move(other.data_)), size_(other.size_) {
        other.size_ = 0;
    }
    
    /**
     * @brief Move assignment operator
     */
    ResourceManager& operator=(ResourceManager&& other) noexcept {
        if (this != &other) {
            data_ = std::move(other.data_);
            size_ = other.size_;
            other.size_ = 0;
        }
        return *this;
    }
    
    /**
     * @brief Deleted copy operations (move-only class)
     */
    ResourceManager(const ResourceManager&) = delete;
    ResourceManager& operator=(const ResourceManager&) = delete;
    
    /**
     * @brief Destructor with cleanup logging
     */
    ~ResourceManager() {
        if (size_ > 0) {
            std::cout << "ResourceManager deallocating " << size_ << " integers\n";
        }
    }
    
    /**
     * @brief Array access operator overload
     */
    int& operator[](size_t index) {
        return data_[index];
    }
    
    /**
     * @brief Const array access operator
     */
    const int& operator[](size_t index) const {
        return data_[index];
    }
    
    size_t size() const noexcept { return size_; }
};

/**
 * @class CustomException
 * @brief Custom exception class demonstration
 */
class CustomException : public std::exception {
private:
    std::string message_;
    
public:
    explicit CustomException(const std::string& msg) : message_(msg) {}
    
    const char* what() const noexcept override {
        return message_.c_str();
    }
};

/**
 * @brief Function demonstrating exception handling and RAII
 * @param risky_operation Flag to trigger exception
 */
void demonstrate_exceptions(bool risky_operation) {
    try {
        ResourceManager manager(10);
        
        if (risky_operation) {
            throw CustomException("Simulated error in operation");
        }
        
        // Use the resource
        for (size_t i = 0; i < manager.size(); ++i) {
            manager[i] = static_cast<int>(i * i);
        }
        
    } catch (const CustomException& e) {
        std::cerr << "Custom exception caught: " << e.what() << std::endl;
    } catch (const std::exception& e) {
        std::cerr << "Standard exception caught: " << e.what() << std::endl;
    }
    // RAII ensures ResourceManager destructor is called
}

/**
 * @brief Function demonstrating lambda expressions and algorithms
 */
void demonstrate_lambdas() {
    std::vector<int> numbers = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    
    // Lambda with capture by value
    int multiplier = 2;
    auto multiply = [multiplier](int x) { return x * multiplier; };
    
    // Lambda with capture by reference
    int sum = 0;
    auto accumulate = [&sum](int x) { sum += x; };
    
    // Generic lambda (C++14)
    auto generic_print = [](const auto& value) {
        std::cout << value << " ";
    };
    
    // Use lambdas with STL algorithms
    std::for_each(numbers.begin(), numbers.end(), accumulate);
    std::cout << "Sum: " << sum << std::endl;
    
    // Transform with lambda
    std::vector<int> doubled;
    std::transform(numbers.begin(), numbers.end(), 
                  std::back_inserter(doubled), multiply);
    
    std::cout << "Doubled: ";
    std::for_each(doubled.begin(), doubled.end(), generic_print);
    std::cout << std::endl;
}

/**
 * @brief Template function demonstrating perfect forwarding
 * @tparam T Type to be forwarded
 * @param value Value to forward
 */
template<typename T>
void perfect_forward(T&& value) {
    // Demonstrate perfect forwarding
    auto process = [](auto&& arg) {
        std::cout << "Processing: " << std::forward<decltype(arg)>(arg) << std::endl;
    };
    process(std::forward<T>(value));
}

/**
 * @brief Main function demonstrating various C++ features
 * @return Program exit code
 */
int main() {
    std::cout << "=== C++ Comprehensive Example ===" << std::endl;
    
    // Polymorphism demonstration
    std::vector<std::unique_ptr<geometry::Shape>> shapes;
    shapes.push_back(std::make_unique<geometry::Circle>(5.0));
    shapes.push_back(std::make_unique<geometry::Rectangle>(4.0, 6.0));
    
    for (const auto& shape : shapes) {
        shape->display();
    }
    
    // Template usage
    std::cout << "Max of 10 and 20: " << utils::max(10, 20) << std::endl;
    std::cout << "Max of 3.14 and 2.71: " << utils::max(3.14, 2.71) << std::endl;
    
    // Container filtering with templates
    std::vector<int> numbers = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    auto even_numbers = utils::filter(numbers, [](int x) { return x % 2 == 0; });
    
    std::cout << "Even numbers: ";
    for (int n : even_numbers) {
        std::cout << n << " ";
    }
    std::cout << std::endl;
    
    // Exception handling demonstration
    demonstrate_exceptions(false);  // Normal operation
    demonstrate_exceptions(true);   // Trigger exception
    
    // Lambda demonstration
    demonstrate_lambdas();
    
    // Perfect forwarding demonstration
    std::string text = "Hello, World!";
    perfect_forward(text);                    // lvalue
    perfect_forward(std::string("Temp"));     // rvalue
    
    // Auto type deduction
    auto automatic_int = 42;
    auto automatic_double = 3.14159;
    auto automatic_string = std::string("auto deduction");
    
    std::cout << "Auto deduced types: " 
              << automatic_int << ", " 
              << automatic_double << ", " 
              << automatic_string << std::endl;
    
    // Range-based for loops
    std::cout << "Numbers: ";
    for (const auto& num : numbers) {
        std::cout << num << " ";
    }
    std::cout << std::endl;
    
    return 0;
}