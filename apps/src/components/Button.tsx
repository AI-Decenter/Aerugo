import React from "react"

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
    variant?: "primary" | "secondary" | "outline";
}

const Button: React.FC<ButtonProps> = ({children, variant = "primary", className, ...props}) => {
    const baseStyles = 'inline-flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-offset-2';
     const variantStyles = {
    primary: 'bg-blue-600 hover:bg-blue-700 text-white focus:ring-blue-500',
    secondary: 'bg-gray-200 hover:bg-gray-300 text-gray-800 focus:ring-gray-500',
    outline: 'bg-white border-gray-300 text-gray-700 hover:bg-gray-50 focus:ring-blue-500',
  };
    return (
        <button className={`${baseStyles}  ${variantStyles[variant]} ${className}`}
        {...props}>
            {children}
        </button>
    )
}

export default Button;