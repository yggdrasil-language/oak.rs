# 🧪 Testing & Verification

Quality is the core of `oaks`. We ensure the stability of the code through multi-dimensional testing.

## 📊 Testing Strategy

### 1. Unit Testing
Fine-grained verification targeted at core functions and edge conditions.

### 2. Integration Testing
Verifies the collaborative work capability between modules, simulating real-world usage scenarios.

### 3. Snapshot Testing (if applicable)
Ensures that output results (such as AST structure) remain consistent with expectations during code iterations.

## 🚀 Running Tests
Execute the following command to start automated testing:
```bash
cargo test
```

## 📈 Contribution Guidelines
If you find an issue in the tests, please:
1. Submit a minimal test case that can reproduce the issue.
2. Ensure that all existing tests still pass after your modifications.
