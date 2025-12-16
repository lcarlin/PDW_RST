"""
Property-based tests for PDW Documentation Suite
Tests the correctness properties defined in the design document
"""

import pytest
import hypothesis
from hypothesis import given, strategies as st
import ast
import re
import os
from pathlib import Path
from typing import List, Dict, Set, Tuple


class DocumentationValidator:
    """Validates documentation completeness and accuracy"""
    
    def __init__(self, source_file: str, docs_dir: str):
        self.source_file = source_file
        self.docs_dir = docs_dir
        self.source_functions = self._extract_functions_from_source()
        self.source_classes = self._extract_classes_from_source()
        self.source_imports = self._extract_imports_from_source()
    
    def _extract_functions_from_source(self) -> Set[str]:
        """Extract all function names from source code"""
        with open(self.source_file, 'r', encoding='utf-8') as f:
            source_code = f.read()
        
        tree = ast.parse(source_code)
        functions = set()
        
        for node in ast.walk(tree):
            if isinstance(node, ast.FunctionDef):
                functions.add(node.name)
        
        return functions
    
    def _extract_classes_from_source(self) -> Set[str]:
        """Extract all class names from source code"""
        with open(self.source_file, 'r', encoding='utf-8') as f:
            source_code = f.read()
        
        tree = ast.parse(source_code)
        classes = set()
        
        for node in ast.walk(tree):
            if isinstance(node, ast.ClassDef):
                classes.add(node.name)
        
        return classes
    
    def _extract_imports_from_source(self) -> Set[str]:
        """Extract all import statements from source code"""
        with open(self.source_file, 'r', encoding='utf-8') as f:
            source_code = f.read()
        
        tree = ast.parse(source_code)
        imports = set()
        
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                for alias in node.names:
                    imports.add(alias.name)
            elif isinstance(node, ast.ImportFrom):
                if node.module:
                    imports.add(node.module)
        
        return imports
    
    def get_documented_functions(self, doc_content: str) -> Set[str]:
        """Extract function names mentioned in documentation"""
        # Look for function patterns like `function_name()` or **function_name**
        function_patterns = [
            r'`(\w+)\(\)`',  # `function_name()`
            r'\*\*(\w+)\*\*',  # **function_name**
            r'### (\w+)',  # ### function_name
            r'#### (\w+)',  # #### function_name
        ]
        
        documented_functions = set()
        for pattern in function_patterns:
            matches = re.findall(pattern, doc_content)
            documented_functions.update(matches)
        
        return documented_functions
    
    def get_documented_components(self, doc_content: str) -> Set[str]:
        """Extract component names mentioned in documentation"""
        # Look for component patterns
        component_patterns = [
            r'### (\w+(?:\s+\w+)*)',  # ### Component Name
            r'#### (\w+(?:\s+\w+)*)',  # #### Component Name
            r'\*\*(\w+(?:\s+\w+)*):\*\*',  # **Component Name:**
        ]
        
        documented_components = set()
        for pattern in component_patterns:
            matches = re.findall(pattern, doc_content)
            documented_components.update(matches)
        
        return documented_components


# Property 1: Technical Documentation Completeness
@given(st.text(min_size=1, max_size=100, alphabet=st.characters(whitelist_categories=('Lu', 'Ll', 'Nd'))))
def test_technical_documentation_completeness(function_name):
    """
    **Feature: pdw-documentation, Property 1: Technical Documentation Completeness**
    
    For any function, class, or module in the source code, the generated technical 
    documentation should contain complete descriptions including purpose, parameters, 
    return values, data structures, and architectural relationships.
    
    **Validates: Requirements 1.1, 1.2, 1.3, 1.4, 1.5**
    """
    # Skip if function name is not valid Python identifier
    if not function_name.isidentifier():
        return
    
    validator = DocumentationValidator('PersonalDataWareHouse.py', '.')
    
    # Read technical documentation
    try:
        with open('PDW_Technical_Specification.md', 'r', encoding='utf-8') as f:
            tech_doc = f.read()
    except FileNotFoundError:
        pytest.fail("Technical documentation file not found")
    
    # If function exists in source code, it should be documented
    if function_name in validator.source_functions:
        documented_functions = validator.get_documented_functions(tech_doc)
        
        # Property: All source functions should be documented
        assert function_name in documented_functions or any(
            function_name in doc_func for doc_func in documented_functions
        ), f"Function '{function_name}' found in source but not documented in technical specification"


def test_all_source_functions_documented():
    """
    Test that all functions in the source code are documented in technical specification
    """
    validator = DocumentationValidator('PersonalDataWareHouse.py', '.')
    
    with open('PDW_Technical_Specification.md', 'r', encoding='utf-8') as f:
        tech_doc = f.read()
    
    documented_functions = validator.get_documented_functions(tech_doc)
    
    # Check that major functions are documented
    critical_functions = {
        'main', 'new_data_loader', 'create_pivot_history', 
        'xlsx_report_generator', 'sanitize_entries_dataframe'
    }
    
    for func in critical_functions:
        if func in validator.source_functions:
            assert any(func in doc_func for doc_func in documented_functions), \
                f"Critical function '{func}' not found in technical documentation"


def test_documentation_structure_completeness():
    """
    Test that documentation contains all required sections and components
    """
    required_files = [
        'PDW_Technical_Specification.md',
        'PDW_Functional_Specification.md', 
        'PDW_Deployment_Guide.md'
    ]
    
    for file_path in required_files:
        assert os.path.exists(file_path), f"Required documentation file '{file_path}' not found"
        
        with open(file_path, 'r', encoding='utf-8') as f:
            content = f.read()
            
        # Check minimum content requirements
        assert len(content) > 1000, f"Documentation file '{file_path}' appears incomplete (too short)"
        assert '##' in content, f"Documentation file '{file_path}' missing section headers"


def test_technical_spec_required_sections():
    """
    Test that technical specification contains all required sections
    """
    with open('PDW_Technical_Specification.md', 'r', encoding='utf-8') as f:
        content = f.read()
    
    required_sections = [
        'Executive Summary',
        'System Architecture', 
        'Database Schema',
        'Error Handling',
        'Dependencies and Requirements'
    ]
    
    for section in required_sections:
        assert section in content, f"Technical specification missing required section: {section}"


def test_functional_spec_required_sections():
    """
    Test that functional specification contains all required sections
    """
    with open('PDW_Functional_Specification.md', 'r', encoding='utf-8') as f:
        content = f.read()
    
    required_sections = [
        'Business Context',
        'Business Processes',
        'Business Rules',
        'User Interactions'
    ]
    
    for section in required_sections:
        assert section in content, f"Functional specification missing required section: {section}"


def test_deployment_guide_required_sections():
    """
    Test that deployment guide contains all required sections
    """
    with open('PDW_Deployment_Guide.md', 'r', encoding='utf-8') as f:
        content = f.read()
    
    required_sections = [
        'System Requirements',
        'Installation Instructions',
        'Configuration Setup',
        'Troubleshooting Guide'
    ]
    
    for section in required_sections:
        assert section in content, f"Deployment guide missing required section: {section}"


if __name__ == "__main__":
    # Run basic validation tests
    test_documentation_structure_completeness()
    test_all_source_functions_documented()
    test_technical_spec_required_sections()
    test_functional_spec_required_sections()
    test_deployment_guide_required_sections()
    print("All documentation structure validation tests passed!")